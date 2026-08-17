//! Owned Frostbite `.layout` document model (ElementTree-style).

use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::path::Path;
use xmltree::{Element, XMLNode};

const CONTAINER_PROP_TYPES: &[&str] = &["object", "struct"];

pub type Area = [f32; 4];

#[derive(Default)]
pub struct LayoutDoc {
    pub root: Option<Element>,
    pub had_bom: bool,
    pub path: Option<std::path::PathBuf>,
    pub dirty: bool,
    pub control_ids: HashSet<String>,
    /// Element identity → (parent object, Children/list prop that owns it). Stored as raw
    /// pointers only while `root` is not mutated structurally without rebuild — we rebuild
    /// parent maps after each structural edit via path keys instead.
    pub class_templates: HashMap<String, Element>,
    pub all_classes: Vec<String>,
}

impl LayoutDoc {
    pub fn load_path(path: &Path) -> Result<Self, String> {
        let raw = std::fs::read(path).map_err(|e| e.to_string())?;
        let had_bom = raw.starts_with(&[0xEF, 0xBB, 0xBF]);
        let data = if had_bom { &raw[3..] } else { &raw[..] };
        let root = Element::parse(Cursor::new(data)).map_err(|e| e.to_string())?;
        let mut doc = Self {
            root: Some(root),
            had_bom,
            path: Some(path.to_path_buf()),
            dirty: false,
            control_ids: HashSet::new(),
            class_templates: HashMap::new(),
            all_classes: Vec::new(),
        };
        doc.reindex();
        Ok(doc)
    }

    pub fn save(&mut self) -> Result<(), String> {
        let path = self
            .path
            .clone()
            .ok_or_else(|| "No file path".to_string())?;
        self.save_as(&path)
    }

    pub fn save_as(&mut self, path: &Path) -> Result<(), String> {
        let root = self.root.as_ref().ok_or_else(|| "No document".to_string())?;
        let mut body = Vec::new();
        root.write(&mut body).map_err(|e| e.to_string())?;
        let mut out = b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n".to_vec();
        // Normalize newlines to CRLF.
        let text = String::from_utf8_lossy(&body);
        for line in text.split('\n') {
            let line = line.trim_end_matches('\r');
            out.extend_from_slice(line.as_bytes());
            out.extend_from_slice(b"\r\n");
        }
        // Avoid trailing double blank from writer + our newline.
        while out.ends_with(b"\r\n\r\n") {
            out.truncate(out.len() - 2);
        }
        if self.had_bom {
            let mut with_bom = vec![0xEF, 0xBB, 0xBF];
            with_bom.append(&mut out);
            out = with_bom;
        }
        std::fs::write(path, out).map_err(|e| e.to_string())?;
        self.path = Some(path.to_path_buf());
        self.dirty = false;
        Ok(())
    }

    pub fn reindex(&mut self) {
        self.control_ids.clear();
        self.class_templates.clear();
        self.all_classes.clear();
        let Some(root) = self.root.as_ref() else {
            return;
        };
        collect_control_ids(root, &mut self.control_ids);
        let mut seen = HashMap::new();
        collect_class_templates(root, &mut seen);
        self.class_templates = seen;
        let mut classes: Vec<_> = self.class_templates.keys().cloned().collect();
        classes.sort();
        self.all_classes = classes;
    }

    pub fn new_control_id(&mut self) -> String {
        let mut rng = rand::thread_rng();
        loop {
            let cid = format!("0x{:08x}", rng.gen::<u32>());
            if self.control_ids.insert(cid.clone()) {
                return cid;
            }
        }
    }
}

pub fn node_label(el: &Element, prop_name: Option<&str>) -> String {
    let cls = el
        .attributes
        .get("cls")
        .map(|s| s.as_str())
        .unwrap_or(el.name.as_str());
    let mut bits = Vec::new();
    if let Some(pn) = prop_name {
        bits.push(format!("{pn}:"));
    }
    bits.push(cls.to_string());
    if let Some(p) = find_prop(el, "Comment") {
        if let Some(v) = p.attributes.get("value") {
            if !v.is_empty() {
                bits.push(format!("\"{v}\""));
            }
        }
    }
    if let Some(p) = find_prop(el, "ControlID") {
        if let Some(v) = p.attributes.get("value") {
            if v != "0" && !v.is_empty() {
                bits.push(format!("[{v}]"));
            }
        }
    }
    bits.join(" ")
}

pub fn find_prop<'a>(el: &'a Element, name: &str) -> Option<&'a Element> {
    for child in &el.children {
        if let XMLNode::Element(p) = child {
            if p.name == "prop" && p.attributes.get("name").map(|s| s.as_str()) == Some(name) {
                return Some(p);
            }
        }
    }
    None
}

pub fn find_prop_mut<'a>(el: &'a mut Element, name: &str) -> Option<&'a mut Element> {
    for child in &mut el.children {
        if let XMLNode::Element(p) = child {
            if p.name == "prop" && p.attributes.get("name").map(|s| s.as_str()) == Some(name) {
                return Some(p);
            }
        }
    }
    None
}

pub fn leaf_props_owned_names(el: &Element) -> Vec<(String, String, Option<String>)> {
    // (name, type, value-or-key)
    let mut out = Vec::new();
    for child in &el.children {
        let XMLNode::Element(p) = child else { continue };
        if p.name != "prop" {
            continue;
        }
        let ptype = p.attributes.get("type").map(|s| s.as_str()).unwrap_or("");
        if CONTAINER_PROP_TYPES.contains(&ptype) {
            continue;
        }
        let name = p
            .attributes
            .get("name")
            .cloned()
            .unwrap_or_default();
        let val = if ptype == "image" {
            p.attributes.get("key").cloned()
        } else if p.attributes.get("count").is_some() {
            let vals: Vec<_> = p
                .children
                .iter()
                .filter_map(|c| match c {
                    XMLNode::Element(v) if v.name == "value" => {
                        Some(v.get_text().unwrap_or_default().into_owned())
                    }
                    _ => None,
                })
                .collect();
            Some(vals.join(", "))
        } else {
            p.attributes.get("value").cloned()
        };
        out.push((name, ptype.to_string(), val));
    }
    out
}

pub fn set_prop_value(el: &mut Element, name: &str, value: &str) -> bool {
    if let Some(p) = find_prop_mut(el, name) {
        let ptype = p.attributes.get("type").cloned().unwrap_or_default();
        if ptype == "image" {
            p.attributes.insert("key".into(), value.to_string());
        } else if p.attributes.get("count").is_some() {
            let parts: Vec<_> = value
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            let mut vi = 0;
            for child in &mut p.children {
                if let XMLNode::Element(v) = child {
                    if v.name == "value" {
                        if let Some(text) = parts.get(vi) {
                            v.children.clear();
                            v.children.push(XMLNode::Text((*text).to_string()));
                        }
                        vi += 1;
                    }
                }
            }
        } else {
            p.attributes.insert("value".into(), value.to_string());
        }
        true
    } else {
        false
    }
}

pub fn get_area(el: &Element) -> Area {
    let Some(p) = find_prop(el, "Area") else {
        return [0.0; 4];
    };
    let Some(v) = p.attributes.get("value") else {
        return [0.0; 4];
    };
    let mut parts = [0.0f32; 4];
    for (i, s) in v.split(',').take(4).enumerate() {
        parts[i] = s.trim().parse().unwrap_or(0.0);
    }
    parts
}

pub fn set_area(el: &mut Element, rect: Area) {
    let Some(p) = find_prop_mut(el, "Area") else {
        return;
    };
    let fmt = |v: f32| {
        if (v - v.round()).abs() < 1e-4 {
            format!("{}", v.round() as i32)
        } else {
            format!("{:.4}", v)
        }
    };
    p.attributes.insert(
        "value".into(),
        format!("{},{},{},{}", fmt(rect[0]), fmt(rect[1]), fmt(rect[2]), fmt(rect[3])),
    );
}

pub fn get_children_prop(el: &Element) -> Option<&Element> {
    for child in &el.children {
        if let XMLNode::Element(p) = child {
            if p.name == "prop"
                && p.attributes.get("name").map(|s| s.as_str()) == Some("Children")
                && p.attributes.get("type").map(|s| s.as_str()) == Some("object")
            {
                return Some(p);
            }
        }
    }
    None
}

pub fn get_children_prop_mut(el: &mut Element) -> Option<&mut Element> {
    for child in &mut el.children {
        if let XMLNode::Element(p) = child {
            if p.name == "prop"
                && p.attributes.get("name").map(|s| s.as_str()) == Some("Children")
                && p.attributes.get("type").map(|s| s.as_str()) == Some("object")
            {
                return Some(p);
            }
        }
    }
    None
}

pub fn ensure_children_prop(el: &mut Element) -> &mut Element {
    if get_children_prop(el).is_some() {
        return get_children_prop_mut(el).unwrap();
    }
    let mut p = Element::new("prop");
    p.attributes.insert("name".into(), "Children".into());
    p.attributes.insert("propid".into(), "0xeec1b00b".into());
    p.attributes.insert("type".into(), "object".into());
    p.attributes.insert("count".into(), "0".into());
    el.children.push(XMLNode::Element(p));
    get_children_prop_mut(el).unwrap()
}

pub fn list_object_items(prop_el: &Element) -> Vec<&Element> {
    prop_el
        .children
        .iter()
        .filter_map(|c| match c {
            XMLNode::Element(e) if e.name == "object" || e.name == "value" => Some(e),
            _ => None,
        })
        .collect()
}

fn collect_control_ids(el: &Element, ids: &mut HashSet<String>) {
    if el.name == "prop" && el.attributes.get("name").map(|s| s.as_str()) == Some("ControlID") {
        if let Some(v) = el.attributes.get("value") {
            if !v.is_empty() {
                ids.insert(v.clone());
            }
        }
    }
    for child in &el.children {
        if let XMLNode::Element(e) = child {
            collect_control_ids(e, ids);
        }
    }
}

fn collect_class_templates(el: &Element, seen: &mut HashMap<String, Element>) {
    if el.name == "object" {
        if let Some(cls) = el.attributes.get("cls") {
            if !seen.contains_key(cls) {
                seen.insert(cls.clone(), el.clone());
            }
        }
    }
    for child in &el.children {
        if let XMLNode::Element(e) = child {
            collect_class_templates(e, seen);
        }
    }
}

/// Path of child indices from document root Element down to a node.
pub type NodePath = Vec<usize>;

pub fn get_at_mut<'a>(root: &'a mut Element, path: &[usize]) -> Option<&'a mut Element> {
    let mut cur = root;
    for &idx in path {
        let child = cur.children.get_mut(idx)?;
        match child {
            XMLNode::Element(e) => cur = e,
            _ => return None,
        }
    }
    Some(cur)
}

pub fn get_at<'a>(root: &'a Element, path: &[usize]) -> Option<&'a Element> {
    let mut cur = root;
    for &idx in path {
        let child = cur.children.get(idx)?;
        match child {
            XMLNode::Element(e) => cur = e,
            _ => return None,
        }
    }
    Some(cur)
}

/// Walk tree for UI: returns (path, prop_label, element_ref-like via path).
pub fn walk_tree(root: &Element, out: &mut Vec<(NodePath, Option<String>)>) {
    for (i, child) in root.children.iter().enumerate() {
        if let XMLNode::Element(obj) = child {
            if obj.name == "object" {
                let path = vec![i];
                out.push((path.clone(), None));
                walk_node(obj, &path, out);
            }
        }
    }
}

fn walk_node(el: &Element, path: &NodePath, out: &mut Vec<(NodePath, Option<String>)>) {
    for (i, child) in el.children.iter().enumerate() {
        let XMLNode::Element(p) = child else { continue };
        if p.name != "prop" {
            continue;
        }
        let ptype = p.attributes.get("type").map(|s| s.as_str()).unwrap_or("");
        let pname = p.attributes.get("name").cloned().unwrap_or_default();
        if ptype == "struct" {
            for (j, c) in p.children.iter().enumerate() {
                if let XMLNode::Element(s) = c {
                    if s.name == "struct" {
                        let mut sp = path.clone();
                        sp.push(i);
                        sp.push(j);
                        out.push((sp.clone(), Some(pname.clone())));
                        walk_node(s, &sp, out);
                    }
                }
            }
        } else if ptype == "object" {
            if p.attributes.get("count").is_some() {
                let mut idx = 0;
                for (j, c) in p.children.iter().enumerate() {
                    if let XMLNode::Element(ch) = c {
                        if ch.name == "object" || ch.name == "value" {
                            idx += 1;
                            let mut sp = path.clone();
                            sp.push(i);
                            sp.push(j);
                            out.push((sp.clone(), Some(format!("{pname}[{idx}]"))));
                            walk_node(ch, &sp, out);
                        }
                    }
                }
            } else {
                for (j, c) in p.children.iter().enumerate() {
                    if let XMLNode::Element(ch) = c {
                        if ch.name == "object" || ch.name == "value" {
                            let mut sp = path.clone();
                            sp.push(i);
                            sp.push(j);
                            out.push((sp.clone(), Some(pname.clone())));
                            walk_node(ch, &sp, out);
                            break;
                        }
                    }
                }
            }
        }
    }
}

pub fn walk_widgets(
    el: &Element,
    path: &NodePath,
    ox: f32,
    oy: f32,
    depth: u32,
    out: &mut Vec<(NodePath, Area, u32)>,
) {
    let area = get_area(el);
    let abs = [ox + area[0], oy + area[1], ox + area[2], oy + area[3]];
    out.push((path.clone(), abs, depth));
    let Some(cp) = get_children_prop(el) else {
        return;
    };
    // Find Children prop index in el.children
    let mut cp_idx = None;
    for (i, child) in el.children.iter().enumerate() {
        if let XMLNode::Element(p) = child {
            if p.name == "prop"
                && p.attributes.get("name").map(|s| s.as_str()) == Some("Children")
            {
                cp_idx = Some(i);
                break;
            }
        }
    }
    let Some(cp_i) = cp_idx else {
        return;
    };
    for (j, child) in cp.children.iter().enumerate() {
        if let XMLNode::Element(obj) = child {
            if obj.name == "object" {
                let mut sp = path.clone();
                sp.push(cp_i);
                sp.push(j);
                walk_widgets(obj, &sp, ox + area[0], oy + area[1], depth + 1, out);
            }
        }
    }
}

pub fn add_child(
    parent: &mut Element,
    template: &Element,
    area: Area,
    new_cid: &str,
) -> NodePath {
    let mut new_el = template.clone();
    if let Some(cid) = find_prop_mut(&mut new_el, "ControlID") {
        cid.attributes.insert("value".into(), new_cid.to_string());
    }
    let cls = new_el
        .attributes
        .get("cls")
        .cloned()
        .unwrap_or_else(|| "Object".into());
    if let Some(comment) = find_prop_mut(&mut new_el, "Comment") {
        comment
            .attributes
            .insert("value".into(), format!("New {cls}"));
    }
    set_area(&mut new_el, area);
    // Strip nested Children from the clone.
    new_el.children.retain(|c| match c {
        XMLNode::Element(p)
            if p.name == "prop" && p.attributes.get("name").map(|s| s.as_str()) == Some("Children") =>
        {
            false
        }
        _ => true,
    });

    let cp = ensure_children_prop(parent);
    cp.children.push(XMLNode::Element(new_el));
    let count = list_object_items(cp).len();
    cp.attributes.insert("count".into(), count.to_string());
    // Caller rebuilds paths — return empty; selection refreshed via re-walk.
    Vec::new()
}

pub fn remove_child_from_list(prop: &mut Element, child_index: usize) -> bool {
    if child_index >= prop.children.len() {
        return false;
    }
    prop.children.remove(child_index);
    let count = list_object_items(prop).len();
    if count == 0 {
        prop.attributes.insert("count".into(), "0".into());
    } else {
        prop.attributes.insert("count".into(), count.to_string());
    }
    true
}

pub fn move_sibling_in_prop(prop: &mut Element, child_index: usize, direction: i32) -> bool {
    let new_idx = child_index as i32 + direction;
    if new_idx < 0 || new_idx as usize >= prop.children.len() {
        return false;
    }
    prop.children.swap(child_index, new_idx as usize);
    true
}

pub fn class_color(cls: &str) -> egui::Color32 {
    match cls {
        "Window" => egui::Color32::from_rgb(0x3a, 0x5a, 0x8c),
        "Button" => egui::Color32::from_rgb(0xc9, 0x8a, 0x3b),
        "Text" => egui::Color32::from_rgb(0x4a, 0x9d, 0x6b),
        "Slider" => egui::Color32::from_rgb(0x9b, 0x59, 0xb6),
        "Mini Map" => egui::Color32::from_rgb(0x2e, 0x8b, 0x8b),
        "ProgressBar" => egui::Color32::from_rgb(0xb0, 0x4a, 0x4a),
        _ => egui::Color32::from_rgb(0x8a, 0x93, 0xa6),
    }
}
