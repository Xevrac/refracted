use crate::blaze::server::BlazeProtocolServer;
use crate::common::game::{current_service_ports, get_current_game};
use crate::lsx::LsxServer;
use crate::qos::QosProtocolServer;
use crate::rtm::RtmProtocolServer;
use crate::web::server::WebProtocolServer;

#[derive(Debug, Clone)]
pub struct ServiceFlags {
    pub web: bool,
    pub blaze: bool,
    pub lsx: bool,
    pub qos: bool,
    pub rtm: bool,
}

impl ServiceFlags {
    pub fn from_current_game() -> Self {
        let Some(g) = get_current_game() else {
            return Self::all_on();
        };
        let on = |name: &str| {
            g.enabled_services
                .iter()
                .any(|s| s.eq_ignore_ascii_case(name))
        };
        Self {
            web: on("Web"),
            blaze: on("Blaze"),
            lsx: on("LSX"),
            qos: on("QoS"),
            rtm: on("RTM"),
        }
    }

    fn all_on() -> Self {
        Self {
            web: true,
            blaze: true,
            lsx: true,
            qos: true,
            rtm: true,
        }
    }
}

/// Ports the emulator will bind for the selected title (`games.json` → `service_ports`).
pub fn aggregated_required_ports() -> Vec<(u16, String)> {
    let f = ServiceFlags::from_current_game();
    let p = current_service_ports();
    let mut ports = Vec::new();
    if f.web {
        ports.extend(WebProtocolServer::ports_from_config(&p));
    }
    if f.blaze {
        ports.extend(BlazeProtocolServer::ports_from_config(&p));
    }
    if f.qos {
        ports.extend(QosProtocolServer::ports_from_config(&p));
    }
    if f.rtm {
        ports.extend(RtmProtocolServer::ports_from_config(&p));
    }
    if f.lsx {
        ports.extend(LsxServer::ports_from_config(&p));
    }
    ports
}
