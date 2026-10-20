/**
 * Learn Controller
 * Handles the Play Tutorial logic and the rich-HTML guide renderer.
 */
CCApp.controller('LearnController', function($scope) {
    
    // ==========================================
    // 1. PLAY TUTORIAL DATA 
    // ==========================================
    // Exact translation from the provided screenshot
    $scope.tutorialData = {
        text: "Welcome to the Command & Conquer tutorial section! Over time, this will become a hub of interactive training tools, designed to help you improve your skills and master Command & Conquer. If you want to try some hands on training, try out the EU Tutorial. We recommend also taking a look at the Beginner's GUIDE in the Learn section Online Beginner's Guide",
        buttonText: "Play EU Tutorial"
    };

    $scope.getTutorialText = function () {
        if (!$scope.supportAurora()) {
            return $scope.tutorialData.text;
        }
        return "Welcome to the Aurora tutorial section! Over time, this will become a hub of interactive training tools, designed to help you improve your skills and master Aurora. If you want to try some hands on training, try out the EU Tutorial. We recommend also taking a look at the guides under Support > Guides.";
    };

    // ==========================================
    // 2. GUIDES DATA (Rich HTML Engine - Angular 1.1.5 Safe)
    // ==========================================
    
    // Sidebar Navigation List (Acts as books/chapters)
    $scope.guideBooks = [
        { id: 'ui', title: "USER INTERFACE" },
        { id: 'basic', title: "BASIC CONTROLS" },
        { id: 'adv', title: "ADVANCED CONTROLS" },
        { id: 'harvest', title: "HARVESTING RESOURCES" },
        { id: 'struct', title: "STRUCTURES" },
        { id: 'prod', title: "UNIT PRODUCTION" },
        { id: 'tech', title: "TECH TREE" },
        { id: 'hotkeys', title: "HOTKEYS" },
        { id: 'custom', title: "CUSTOMIZE" },
        { id: 'modes', title: "GAME MODES" },
        { id: 'tweaks', title: "TWEAKS", aurora: true }
    ];

    /** Aurora-only books (aurora: true) are hidden in the original 2013 support view. */
    $scope.guideBookVisible = function (book) {
        return !book.aurora || $scope.supportAurora();
    };

    $scope.activeGuideId = 'ui'; // Default active book

    // Structured JSON representing all guide documents
    var structuredGuideDatabase = {
        'ui': {
            title: "USER INTERFACE",
            blocks: [
                { type: "p", content: "The User Interface is what you will use to Build and Command your Units and Structures." },
                { type: "image-wrap", src: "images/learn/Interface_Scr01.jpg", style: "width:100%; ", alt: "User Interface Map Overview" },
                { type: "h3", content: "Minimap / Radar" },
                { type: "p", content: "Shows Environment Layout as well as Unit and Structure Locations<br>You can also:" },
                { type: "list", items: [
                    "Jump to a specific location by Left-Clicking on it on the Minimap",
                    "View your Structure Power bar",
                    "View your Gold",
                    { text: "Toggle House Color with the AllianceButton", subItems: [
                        "When toggled, the Player's house color will be Green.",
                        "Allied Players' house color will be Yellow",
                        "Enemy Players' house color will be Red"
                    ]},
                    "Use the Ping Button to send alerts to your teammates"
                ]},
                
                { type: "h3", content: "Selection Panel" },
                { type: "p", content: "Shows Selected Units as well as Structures.<br>You can also:" },
                { type: "list", items: [
                    { text: "View a single selected Unit/Structure, as well as its Health:", image: { src: "images/learn/Guide_Sel_01.png", style: "margin: 10px 0; ", alt: "Single Selection" } },
                    { text: "View multiple selected Units/Structures and their Health", image: { src: "images/learn/Guide_Sel_02.png", style: "margin: 10px 0; ", alt: "Multiple Selection" } },
                    "Left-Click on a Unit/Structure to de-select everything except that Unit/Structure"
                ]},
                
                { type: "h3", content: "Short-cut Tabs" },
                { type: "p", content: "Displays a short-cut tab for every Production Structure and/or Construction Unit.<br>You can also:" },
                { type: "list", items: [
                    "Select a specific Production Structure or Construction Unit by left-clicking the tab",
                    "Double-click any tab and it will take you to that Production Structure or Construction Unit on the map"
                ]},
                
                { type: "h3", content: "Contextual Actions Panel" },
                { type: "p", content: "Allows you to access the Contextual Actions of specific Units or Structures<br>You can also:" },
                { type: "list", items: [
                    "Build structures from a construction unit.",
                    "Build units from a production structure.",
                    "Access unit abilities of some combat units."
                ]},
                
                { type: "h3", content: "Generals Panel" },
                { type: "p", content: "Displays the available Player Powers, the Player's current ranking, and progress bar toward your next ranking." },
                { type: "list", items: [
                    "Your General can level up by building units and structures and killing enemies",
                    "Your General can reach a maximum ranking of 5.",
                    "As you level up, you will unlock Player Powers specific to your General"
                ]},
                
                { type: "h3", content: "Generals Portrait" },
                { type: "p", content: "General's portrait at the top of the screen will display the current Generals in-game for both allied and enemy players. The house color for the General in game is displayed at the bottom of the portrait and the current general level displayed as stars on the portrait." },

                { type: "h3", content: "Connection & Performance Warnings" },
                { type: "p", content: "When something is getting in the way of a smooth game, a warning icon appears on the right-hand side of the screen, near the top. Nothing is shown while everything is running well, and each icon goes away by itself once things recover.<br>An <b style=\"color:#F2A93B;\">amber</b> icon is a warning. A <b style=\"color:#EE4B2B;\">red</b> icon means the problem is serious. If more than one problem happens at once, the icons stack one below the other." },
                { type: "table", headers: [{content: "Icon", style: "width: 120px;"}, "What it means", "What you can do"], rows: [
                    [{ type: "html", style: "padding: 10px; width: 120px; white-space: nowrap;", content: "<img src=\"images/learn/Guide_Net_LatencyWarning.png\" alt=\"\" style=\"width:48px; height:48px; margin-right:6px;\"><img src=\"images/learn/Guide_Net_LatencySevere.png\" alt=\"\" style=\"width:48px; height:48px; margin-right:6px;\">" },
                     "<b>High Ping</b><br>Your orders are taking a long time to reach the server and come back, so units may react late when you command them.<br>Amber: ping over 100 ms. Red: ping over 200 ms.",
                     { type: "list", items: ["Pause downloads, streaming or game updates", "Use a wired connection instead of Wi-Fi", "Play on the realm closest to you"] }],
                    [{ type: "html", style: "padding: 10px; width: 120px; white-space: nowrap;", content: "<img src=\"images/learn/Guide_Net_JitterWarning.png\" alt=\"\" style=\"width:48px; height:48px; margin-right:6px;\"><img src=\"images/learn/Guide_Net_JitterSevere.png\" alt=\"\" style=\"width:48px; height:48px; margin-right:6px;\">" },
                     "<b>Unstable Connection</b><br>Your ping keeps jumping up and down instead of staying steady, so units may move or respond unevenly even when your average ping looks fine. This is only checked once your ping is over 100 ms, because smaller swings do not affect the game.",
                     { type: "list", items: ["Move closer to your router or use a cable", "Stop other devices on your network from streaming or downloading", "Restart your router if it keeps happening"] }],
                    [{ type: "html", style: "padding: 10px; width: 120px; white-space: nowrap;", content: "<img src=\"images/learn/Guide_Net_FramerateWarning.png\" alt=\"\" style=\"width:48px; height:48px; margin-right:6px;\"><img src=\"images/learn/Guide_Net_FramerateSevere.png\" alt=\"\" style=\"width:48px; height:48px; margin-right:6px;\">" },
                     "<b>Low Framerate</b><br>Your computer is struggling to draw the game smoothly. This comes from your PC, not your connection.<br>Amber: under 30 frames per second. Red: under 20.",
                     { type: "list", items: ["Lower the graphics quality in Options > Graphics", "Close other programs running in the background", "Make sure your graphics drivers are up to date"] }],
                    [{ type: "html", style: "padding: 10px; width: 120px; white-space: nowrap;", content: "<img src=\"images/learn/Guide_Net_ServerWarning.png\" alt=\"\" style=\"width:48px; height:48px; margin-right:6px;\"><img src=\"images/learn/Guide_Net_ServerSevere.png\" alt=\"\" style=\"width:48px; height:48px; margin-right:6px;\">" },
                     "<b>Server Falling Behind</b><br>The game server cannot keep up with the match, so units may pause and then jump forward for everyone in the game. This comes from the server, not your PC or your connection.<br>Amber: the server is running below 90% of normal speed. Red: below 70%.",
                     { type: "list", items: ["If you host the server, lower its tick rate to 30 Hz (or 10 Hz) in the launcher's Servers tab", "If you host the server, close other heavy programs on that computer", "If someone else hosts it, let them know or try another server"] }]
                ]},

                { type: "h3", content: "Network Ping Display" },
                { type: "p", content: "If you would like to see your ping all the time, turn on <b>Options > Interface > Advanced > Display Network Ping (ms)</b>. Your ping is then shown in milliseconds (ms) just below the match timer at the top of the screen. Lower is better: anything under 100 ms is good." }
            ]
        },
        'basic': {
            title: "BASIC CONTROLS",
            blocks: [
                { type: "h3", content: "CAMERA" },
                { type: "p", content: "There are several ways to move the Camera through the Game Environment" },
                { type: "list", items: [
                    "Arrow Keys",
                    "Move the Cursor to the appropriate edge of the screen: it will scroll in that direction",
                    "Left-Click on the Minimap / Radar (left area of UI) to jump to a specific location",
                    "Use the middle Mouse Wheel to move the camera in and out",
                    "Hold the middle mouse wheel, then move the Mouse in the direction you wish to move the camera",
                    "Camera controls can be switched to the Right Mouse Button in the Options menu"
                ]},
                { type: "h3", content: "SELECTING UNITS OR STRUCTURES" },
                { type: "p", content: "Selecting a Unit or Structure provides you with valuable information." },
                { type: "list", items: [
                    "Left-Click to select a single Unit or Structure",
                    "Hold the Left Mouse Button, drag and release to Drag-Select multiple Units or Structures<br>When a Unit is Selected, it:",
                    "Displays a green Health Bar directly above itself,",
                    "Appears in the Selection Panel (center area of UI), and",
                    "Displays its abilities in the Contextual Actions Panel (right area of UI)"
                ]},
                { type: "h3", content: "MOVEMENT" },
                { type: "p", content: "To move Units:" },
                { type: "list", items: [
                    "Select the Units you wish to move, and...",
                    "Right-Click on the location you wish to move them to.",
                    "If you want your Units to automatically pause and attack any enemies on the way to their destination (Attack Move): press A before you right click. You can click anywhere on your current screen in the game world or click on the Minimap"
                ]},
                { type: "h3", content: "COMBAT" },
                { type: "p", content: "Attacking" },
                { type: "list", items: [
                    "Units that are Idle will automatically attack any Enemy Units within their vision range.",
                    "Turreted Units will automatically attack Enemy Units while moving.",
                    { text: "If you want to use specific Units to attack specific Enemy Units or Structures:", subItems: [
                        "Select the Units you wish to use, and",
                        "Right-Click on the chosen target."
                    ]},
                    "When a Unit is attackable, the Mouse Cursor will become an \"Attack Cursor.\""
                ]},
                { type: "p", content: "Defending" },
                { type: "list", items: [
                    { text: "Infantry can Garrison Civilian Structures to improve their longevity and attack range.", subItems: [
                        "Select the Infantry you wish to use,",
                        "Right-Click on your chosen Civilian Structure to garrison your units, and",
                        "Mouse over the Civilian Structure to see how many \"Garrison Slots\" are available."
                    ]},
                    { text: "To remove the Units from their Garrison, select the Civilian Structure and click on the de-garrison button.", image: { src: "images/learn/Garrisondegarrison.png", style: "width:100px; height:100px; margin: 10px 0; ", alt: "De-garrison button" } }
                ]},
                { type: "p", content: "Veterancy" },
                { type: "list", items: [
                    "Units can gain veterancy by achieving a set amount of kills in combat.",
                    { text: "Once a unit has reached the needed amount of kills to gain veterancy, the unit will gain bonuses based on the veterancy rank.", subItems: [
                        { text: "Rank 1", subItems: ["+15% health"] },
                        { text: "Rank 2", subItems: ["+5 HP/s regen"] },
                        { text: "Rank 3", subItems: ["+10% attack rate", "or +1 armor (for units that do not have direct attacks)"] }
                    ]},
                    "The bonuses granted by rank will stack with one another so a max veterancy unit will have all the perks from ranks 1 through 3."
                ]},
                { type: "p", content: "Fog of War" },
                { type: "list", items: [
                    "Upon entering the map, unexplored areas are covered by the Fog of War; this is represented by a dark shroud.",
                    "Enemies within the shroud cannot be seen on the minimap or in-game by the player.",
                    "Enemy structures can be seen underneath the shroud. Constructing a Command Center will prevent structures from being visible beneath the shroud.",
                    "When a unit enters into unexplored areas, the Fog of War is lifted and the player can see enemy units and structures within that unit's field of vision.",
                    "If unit moves out of an area, the shroud will return. To indicate that an area has been previously explored, the shroud will be lighter than unexplored areas.",
                    "Ground units are unable to see up cliffs and will not reveal the shroud up a cliff until they move on top of the cliff.",
                    "Air units can see up cliffs and will remove shroud both above and below cliffs.",
                    "Offensive player powers cannot be cast into the shroud. Superweapons and Vision player powers can be cast into the shroud."
                ]}
            ]
        },
        'adv': {
            title: "ADVANCED CONTROLS",
            blocks: [
                { type: "h3", content: "CONTROL GROUPS" },
                { type: "list", items: [
                    { text: "You can assign a group of Units or Structures to a number key:", subItems: [
                        "Select the Units or Structures you wish to be grouped, and",
                        "Press CTRL-(number key) e.g. CTRL-1",
                        "The grouped Units will display the group number on their top-left side."
                    ]},
                    { text: "From then on:", subItems: [
                        "Pressing that number key will select that group.",
                        "Pressing the number key twice in quick succession will jump the Camera to that group"
                    ]},
                    "Units and Structures can be in multiple Control Groups."
                ]}
            ]
        },
        'harvest': {
            title: "HARVESTING RESOURCES",
            blocks: [
                { type: "p", content: "You will need to obtain resources in order to expand and lead your Infantry to Victory. You gain resources by Harvesting Gold from Resource Centers via your Supply Center or through Oil Wells. Each faction begins with a small force capable of gathering income and eventually creating a massive army." },
                { type: "table", tableStyle: "width:100%; text-align:left; border-collapse: collapse; margin: 20px 0;", headers: ["Faction", "Starting Units and Structures"], rows: [
                    ["APA", { type: "list", items: ["2 Harvesters", "1 Crane", "1 Supply Centers"] }],
                    ["EU", { type: "list", items: ["1 Harvesters", "1 Dozer", "1 Supply Centers"] }],
                    ["GLA", { type: "list", items: ["4 Workers (can be used for harvesting or construction)", "1 Supply Centers"] }]
                ]},
                { type: "h3", content: "GOLD" },
                { type: "image-panel", label: "Harvesters<br>(APA, EU, GLA)", src: ["images/learn/Gen2_APA_Supply_Truck.png", "images/learn/EU_Harvester_Portrait.png", "images/learn/Gen2_Worker_Portrait.png"], alt: "Harvesters" },
                { type: "image-panel", label: "Resource Depot", src: "images/learn/Ns_resourcecenter_portrait.png", alt: "Resource Depot" },
                { type: "p", content: "Resource Depot" },
                { type: "list", items: [
                    "Gold is required to build Units and Structures",
                    "To gather Gold, select your Harvester and right-click on your base's Resource Depot",
                    "Resource Depots will eventually run out of money. Scout the map to find new ones.",
                    "Fastest mining from a Resource Depot occurs with an optimal number of Harvesters.",
                    { text: "For optimal income:", subItems: [
                        "EU requires 3 Harvesters",
                        "APA requires 5 Harvesters",
                        "GLA requires 8 Harvesters"
                    ]}
                ]},
                { type: "h3", content: "TIP" },
                { type: "image-panel", label: "Supply Centers<br>(APA, EU, GLA)", src: ["images/learn/AS_SupplyCenter_Portrait.png", "images/learn/ES_SupplyCenter_Portrait.png", "images/learn/GS_SupplyCenter_Portrait.png"], alt: "Supply Centers" },
                { type: "p", content: "If you want to speed up your income:" },
                { type: "list", items: [
                    "Select your Supply Center either in the game world or via the Shortcut Tabs",
                    "Build another Harvester from your Supply Center (up to the optimal Harvester count per your faction), and",
                    "Select it and right click on the same Resource Center or",
                    "Try expanding to another base by building another Supply Center near a new Resource Center"
                ]},
                { type: "h3", content: "OIL" },
                { type: "image-panel", label: "Oil Derricks<br>(APA, EU, GLA)", src: ["images/learn/AS_OilDerrick_Portrait.png", "images/learn/ES_OilDerrick_Portrait.png", "images/learn/GS_OilDerrick_Portrait.png"], alt: "Oil Derricks" },
                { type: "image-panel", label: "Oil Well", src: "images/learn/Global_oilwell_portrait.png", alt: "Oil Well" },
                { type: "p", content: "Oil Well:" },
                { type: "list", items: [
                    "Oil wells are an alternative means of gathering Gold.",
                    "Oil Wells are found throughout the battlefield.",
                    "To utilize Oil wells, build the Oil Derrick Structure on top of an Oil Well.",
                    "Captured Oil Wells provide a set amount of Gold every few seconds, without the need for a Harvester.",
                    "Oil wells are not unlimited and will eventually be depleted of resources."
                ]}
            ]
        },
        'struct': {
            title: "STRUCTURES",
            blocks: [
                { type: "h3", content: "CONSTRUCTION" },
                { type: "image-panel", label: "Construction Units<br>(APA, EU, GLA)", src: ["images/learn/AU_ConstructionUnit_Portrait.png", "images/learn/EU_ConstructionUnit_Portrait.png", "images/learn/Gen2_Worker_Portrait.png"], alt: "Construction Units" },
                { type: "list", items: [
                    "Select your Construction Unit, either in the game world or via the Shortcut Tabs, and",
                    "Select the Structure you wish to build from the Contextual Actions Panel.",
                    "If you cannot build the Structure, you will receive a message telling you why you are unable to build it. Note: Certain Units and Structures require a Tech Structure before they can be built. These will be greyed out in the Contextual Action Panel. See Tech Tree for more information.",
                    "A \"Ghost\" version of the structure will appear attached to your Cursor.",
                    { text: "Move the Ghost Structure to your chosen location and left-click.", subItems: [
                        "If you cannot place the Structure in an area, the \"Ghost\" will turn red."
                    ]},
                    "The Construction Unit will move to your chosen location and build the Structure.",
                    { text: "The Build icon is overlaid with a dark \"Countdown Timer.\"", subItems: [
                        "The Countdown Timer gradually reveals the icon behind",
                        "The Structure is complete when the Countdown Timer is no longer visible."
                    ]}
                ]},
                { type: "h3", content: "POWER" },
                { type: "image-panel", label: "Power Plants<br>(APA, EU)", src: ["images/learn/AS_PowerPlant_Portrait.png", "images/learn/ES_PowerPlant_Portrait.png"], alt: "Power Plants" },
                { type: "list", items: [
                    "APA and EU Structures require Power to be functional. No power is required for GLA.",
                    "Power is provided by different structures depending on your starting faction."
                ]},
                { type: "table", tableStyle: "width:100%; text-align:left; border-collapse: collapse; margin: 20px 0;", headers: [{content: "Faction", style: "width: 10%;"}, "Power Requirements"], rows: [
                    ["APA", { type: "html", content: '<ul><li>Power Plants generate power, placing certain structures drain power out of the currently generated power.</li><li>There is no defined radius for Power Plants, and structures do not need to be placed near the Power Plants.</li><li>Total Structure Power is displayed at the left of the Minimap.</li></ul><p>If the power requirement of the placed structures exceeds the generated power, your base will enter a Low Power State.</p><ul><li>Unit Construction is 33% slower.</li><li>Structure Construction is 33% slower.</li><li>Base Defense will no longer function.</li></ul>' }],
                    ["EU", { type: "html", content: '<ul><li>Power Plants generate power, placing certain structures drain power out of the currently generated power.</li><li>There is no defined radius for Power Plants, and structures do not need to be placed near the Power Plants.</li><li>Total Structure Power is displayed at the left of the Minimap.</li></ul><p>If the power requirement of the placed structures exceeds the generated power, your base will enter a Low Power State.</p><ul><li>Unit Construction is 33% slower.</li><li>Structure Construction is 33% slower.</li><li>Base Defense will no longer function.</li></ul>' }],
                    ["GLA", { type: "html", content: '<ul><li>There are no Power Plants nor are there any requirements for power, as all buildings and vehicles are constructed by individual workers.</li></ul>' }]
                ]}
            ]
        },
        'prod': {
            title: "UNIT PRODUCTION",
            blocks: [
                { type: "image-wrap", src: "images/learn/ES_BarracksAdvanced_Portrait.png", wrapStyle: "text-align: left; margin: 15px 0;", style: "max-width: 100%; border: none;", alt: "Production Structure" },
                { type: "list", items: [
                    "Select your Production Structure - Barracks etc. - either in the game world or via the Shortcut Tabs.",
                    "The Units that can be trained from that Structure will appear in the Contextual Actions Panel.",
                    { text: "Select the Unit you wish to train.", subItems: [
                        "You can queue up to 9 Units per Structure.",
                        "The Training Queue is indicated by a number overlaid on the respective Unit icons.",
                        "If you cannot build your chosen Unit, an error message will tell you why you are unable to train the Unit.",
                        "You must have the full funds required on hand to train a unit. If insufficient funds are present the unit will not be added to the build queue. Note: Certain Units and Structure require a Tech Structure before they can be trained - these will be greyed out in the Contextual Action Panel. See Tech Tree for details."
                    ]},
                    "You can cancel a Unit by Right-Clicking on their Train Icon.",
                    { text: "The Unit Icon is overlaid with a dark \"Countdown Timer.\"", subItems: [
                        "The Countdown Timer gradually reveals the icon behind it.",
                        "The Unit is complete when the Countdown Timer is no longer visible."
                    ]},
                    "Once the Icon is no longer dark, the Unit will emerge from the Production Structure.",
                    "Rally Points: If you select your Production Structure and right click on the ground, all future Units trained from that Structure will move to that location - or Rally Point - once trained."
                ]}
            ]
        },
        'tech': {
            title: "TECH TREE",
            blocks: [
                { type: "list", items: [
                    "Certain Structures are required to unlock more advanced Units or Structures",
                    "Refer to the on-screen text to determine which structures are pre-requisites for other structures"
                ]}
            ]
        },
        'hotkeys': {
            title: "HOTKEYS",
            blocks: [
                { type: "table", tableStyle: "width:100%; text-align:left; border-collapse: collapse; margin: 15px 0; background-color: rgba(30, 30, 30, 0.5); border: 1px solid #0a0a0a;", hideHeaders: true, rows: [
                    [{ content: "A", style: "padding: 15px 20px; width: 20%; color: #fff; border-right: 1px solid #0a0a0a;" }, { content: "Attack Move", style: "padding: 15px 20px; color: #fff;" }],
                    [{ content: "=", style: "padding: 15px 20px; width: 20%; color: #fff; border-right: 1px solid #0a0a0a;", trStyle: "border-top: 1px solid #0a0a0a;" }, { content: "Toggle Full-Screen", style: "padding: 15px 20px; color: #fff;" }],
                    [{ content: "Control + Click", style: "padding: 15px 20px; width: 20%; color: #fff; border-right: 1px solid #0a0a0a;", trStyle: "border-top: 1px solid #0a0a0a;" }, { content: "Units will bypass any enemies that they encounter and move to the specified target", style: "padding: 15px 20px; color: #fff;" }],
                    [{ content: "CTRL-(1-9)", style: "padding: 15px 20px; width: 20%; color: #fff; border-right: 1px solid #0a0a0a;", trStyle: "border-top: 1px solid #0a0a0a;" }, { content: "Assign Units and / or Structures to a Control Group", style: "padding: 15px 20px; color: #fff;" }],
                    [{ content: "1-9", style: "padding: 15px 20px; width: 20%; color: #fff; border-right: 1px solid #0a0a0a;", trStyle: "border-top: 1px solid #0a0a0a;" }, { content: "Single-Press selects the Control Group. Double-Press jumps the Camera to their location", style: "padding: 15px 20px; color: #fff;" }]
                ]}
            ]
        },
        'custom': {
            title: "CUSTOMIZE",
            blocks: [
                { type: "h3", content: "General unlocks" },
                { type: "list", items: [
                    "Additional Generals are able to be unlocked through the Customize section of the Main Menu.",
                    "Each General has a specialty that grants them unique units, player powers and abilities.",
                    "CP is required to unlock Generals. If you do not have enough CP to purchase a General, you will be unable to unlock them.",
                    "Players start with 1 General for each faction when first entering Command & Conquer."
                ]},
                { type: "h3", content: "Faction Level" },
                { type: "list", items: [
                    "Faction Level is your current level for a specific faction that is increased through use of that faction through playing game modes.",
                    "Experience for a faction can be earned by playing any game mode.",
                    "At the end of a game, experience will be added to a pool for the faction that you completed the game with.",
                    "Upon attaining certain levels of experience with a faction, additional skill will be made available for purchase."
                ]}
            ]
        },
        'modes': {
            title: "GAME MODES",
            blocks: [
                { type: "h3", content: "General Selection" },
                { type: "list", items: [
                    "Players can select their General from the main menu.",
                    "Once a player has found a match, they are unable to switch their selected General.",
                    "Click on the portrait in the upper left hand corner of the main menu to select the general you wish to play as.",
                    "Once selected a green border will appear around the General's portrait to denote that it is the selected General."
                ]},
                { type: "h3", content: "General Modes" },
                { type: "list", items: [
                    "Players can select their game mode from the dropdown next to the PLAY button on the main menu."
                ]},
                { type: "h3", content: "Premium Trial" },
                { type: "list", items: [
                    "Players upon starting the game are given a Premium Trial which give the player access to additional tools to customize game modes."
                ]},
                { type: "h3", content: "Earning CP" },
                { type: "list", items: [
                    "CP is the currency used to unlock skills and Generals.",
                    "CP can be earned by playing any game mode, win or lose.",
                    "At the end of a match, you will be informed how much CP they have gained."
                ]}
            ]
        }
    };

    // Aurora view of the books that describe services Aurora does not have (CP, Premium, Customize).
    var auroraGuideOverrides = {
        'tweaks': {
            title: "TWEAKS",
            blocks: [
                { type: "p", content: "Tweaks are optional match rules the lobby host sets before the battle. Everyone in the lobby plays by the same tweaks, and the lobby shows when the host has changed anything from the defaults." },

                { type: "h3", content: "Presets" },
                { type: "p", content: "A preset is a named set of tweaks. Picking a preset sets every tweak at once; changing a tweak by hand turns the lobby into <b>Custom tweaks</b> until it matches a preset again." },
                { type: "table", headers: [{ content: "Preset type", style: "width: 180px;" }, "What it is"], rows: [
                    ["<b>Generals 2</b>", "Built in. The 2013 Alpha, also known as Generals 2, faithfully restored. Every lobby starts on it unless you choose another default."],
                    ["<b>Generals Classic</b>", "Built in. Turns on the rules and gameplay changes inspired by Generals and Zero Hour."],
                    ["<b>Your presets</b>", "Saved by you with <b>Save as preset</b>. Up to 12, each with a name of up to 24 characters. They are kept on this PC, so they follow you into every lobby you host here."],
                    ["<b>Default preset</b>", "The preset whose star is lit. Every lobby you set up opens on it. Built-in presets cannot be deleted; deleting your default preset makes Generals 2 the default again."]
                ]},

                { type: "h3", content: "The tweaks" },
                { type: "table", headers: [{ content: "Tweak", style: "width: 180px;" }, "What it does", { content: "Generals 2", style: "width: 90px;" }, { content: "Generals Classic", style: "width: 90px;" }], rows: [
                    ["<b>Instant Selling</b>", "Selling a structure completes almost instantly instead of taking 10 seconds.", "Off", "Off"],
                    ["<b>Rebuildable Derricks</b>", "A destroyed oil derrick leaves its oil well behind, so you can build a new derrick on it.", "On", "Off"],
                    ["<b>Combat Tuning (Classic)</b>", "Generals-style armor: damage depends on the weapon type against the armor type. Replaces the game's bonus damage. See below.", "Off", "On"]
                ]},
                { type: "p", content: "Tech Tree, Special Abilities and Factions Only will be added as tweaks soon." },

                { type: "h3", content: "Combat Tuning (Classic)" },
                { type: "p", content: "With this tweak off, each weapon deals its normal damage plus a bonus against the armor types it is built to fight (for example, a rocket soldier's bonus against heavy armor). With it on, damage works as in Generals: every weapon belongs to a <b>weapon type</b>, every unit or structure has an <b>armor type</b>, and the weapon deals a set percentage of its damage against that armor. The unit's <b>armor rating</b> (0 to 8) then lowers the damage by a further 4% per point, so the toughest armor takes 68%." },
                { type: "table", headers: [{ content: "Weapon type", style: "width: 180px;" }, "Infantry", "Light", "Heavy", "Structure", "Spec Ops"], rows: [
                    ["<b>Small arms</b><br>rifles, machine guns, gatling guns, autocannons, pistols, shotguns", "100%", "50%", "20%", "25%", "75%"],
                    ["<b>Sniper</b>", "100%", "10%", "5%", "5%", "100%"],
                    ["<b>Armor-piercing</b><br>tank cannons, railguns, lasers", "25%", "100%", "100%", "75%", "50%"],
                    ["<b>Explosive</b><br>rockets, RPGs, bombs, howitzers, grenades, flak", "35%", "100%", "100%", "100%", "75%"],
                    ["<b>Flame</b><br>flamethrowers, napalm, molotovs", "150%", "50%", "25%", "100%", "100%"],
                    ["<b>Toxin</b><br>toxin weapons, radiation", "150%", "50%", "20%", "25%", "100%"],
                    ["<b>EMP</b>", "25%", "100%", "100%", "50%", "25%"]
                ]},
                { type: "p", content: "Example: a Ranger squad's rifles deal about 2 damage per volley to an APA heavy cannon tank, while an EU rocket squad deals about 41. Bring the right weapon for the target." },

                { type: "h3", content: "Always on: props and clear shots" },
                { type: "list", items: [
                    "Fences, walls, barrels, crates and road barriers block shots, but break quickly: one rocket, shell or bomb destroys them, and any other weapon needs four hits. This is the same whatever tweaks are set.",
                    "If you order a unit to attack something behind a building, it moves around the building to get a clear shot instead of firing into the wall. If there is no way round, it fires at the building."
                ]},

                { type: "h3", content: "Interface: the lobby options box" },
                { type: "p", content: "The box under Team 2 sums up the lobby's rules. It reads <b>Vanilla game, default options set.</b> when nothing has changed:" },
                { type: "image-wrap", src: "images/learn/Guide_Tweaks_Summary_Vanilla.png", style: "max-width:100%;", alt: "Lobby options box, default options" },
                { type: "p", content: "When tweaks are changed it shows a spanner and the preset name. The host clicks the box to open <b>Lobby Options</b>; other players click it to see what the host changed." },
                { type: "image-wrap", src: "images/learn/Guide_Tweaks_Summary_Custom.png", style: "max-width:100%;", alt: "Lobby options box, Generals Classic preset" },
                { type: "image-wrap", src: "images/learn/Guide_Tweaks_Lobby_Vanilla.jpg", style: "width:100%;", alt: "Skirmish lobby with the lobby options box at the bottom right" },

                { type: "h3", content: "Interface: the Lobby Options window" },
                { type: "image-wrap", src: "images/learn/Guide_Tweaks_Options_Window.png", style: "max-width:100%;", alt: "Lobby Options window" },
                { type: "list", items: [
                    "<b>Match password</b>: type a password and click <b>Set</b> to make the lobby private. Leave it blank for an open lobby.",
                    "<b>Disallow Spectators</b>: closes the spectator seat.",
                    "<b>Preset</b>: click a preset to apply it. The highlighted row is the active preset, and the line underneath describes it.",
                    "<b>Star</b>: makes that preset your default for lobbies you set up.",
                    "<b>&times;</b>: deletes one of your own presets. The built-in presets cannot be deleted.",
                    "<b>Tweaks</b>: click to show or hide the list of tweaks, each with a short description.",
                    "<b>Done</b>: closes the window. Your changes are already applied."
                ]},

                { type: "h3", content: "Interface: changing tweaks" },
                { type: "p", content: "Open <b>Tweaks</b> and tick or untick a box. Each tweak has a one-line description under its name, and this page has the full details:" },
                { type: "image-wrap", src: "images/learn/Guide_Tweaks_Tweaks_Open_Hover.jpg", style: "width:100%;", alt: "Tweaks list with a description under each tweak" },
                { type: "p", content: "Type in <b>Search tweaks</b> to filter the list by name or description:" },
                { type: "image-wrap", src: "images/learn/Guide_Tweaks_Tweaks_Search.png", style: "max-width:100%;", alt: "Searching tweaks for derrick" },
                { type: "p", content: "When your tweaks no longer match a preset, no preset is highlighted and the lobby reads <b>Custom tweaks, not saved as a preset.</b>" },
                { type: "image-wrap", src: "images/learn/Guide_Tweaks_Preset_Custom_Mix.png", style: "max-width:100%;", alt: "Custom tweaks, not saved as a preset" },
                { type: "p", content: "Every change is sent to the lobby straight away. Everyone except the host is un-readied, and the lobby chat shows <b>Notice: Host has modified lobby options!</b> so players can review the new rules." },

                { type: "h3", content: "Interface: saving a preset" },
                { type: "list", items: [
                    "Set the tweaks you want, then click <b>Save as preset</b>.",
                    "Type a name and click <b>Save</b>. Saving with the name of one of your presets replaces it; the names of the built-in presets are taken.",
                    "Click the star on your new preset to open every lobby you set up with it."
                ]},
                { type: "image-wrap", src: "images/learn/Guide_Tweaks_Save_Preset.png", style: "max-width:100%;", alt: "Naming a new preset" },
                { type: "image-wrap", src: "images/learn/Guide_Tweaks_Preset_Saved_Default.png", style: "max-width:100%;", alt: "Saved preset set as default" },

                { type: "h3", content: "Interface: for players who are not the host" },
                { type: "p", content: "Click the lobby options box to see the active preset and every option the host changed, with its default value:" },
                { type: "image-wrap", src: "images/learn/Guide_Tweaks_Rules_Dialog.png", style: "max-width:100%;", alt: "Custom game dialog" }
            ]
        },
        'custom': {
            title: "CUSTOMIZE",
            blocks: [
                { type: "h3", content: "Generals" },
                { type: "list", items: [
                    "Each General has a specialty that grants them unique units, player powers and abilities.",
                    "Pick your General on your slot in the lobby, next to your Faction.",
                    "Set <b>Preferred Faction</b> and <b>Preferred General</b> under <b>Options > Gameplay > Lobby Defaults</b> to start every new lobby with them.",
                    "Customising Generals and loadouts is coming soon under <b>Profile > Generals</b>."
                ]}
            ]
        },
        'modes': {
            title: "GAME MODES",
            blocks: [
                { type: "h3", content: "General Selection" },
                { type: "list", items: [
                    "Choose your General on your slot in the lobby before the battle starts.",
                    "The main menu shows the General you played last."
                ]},
                { type: "h3", content: "Find a match" },
                { type: "list", items: [
                    "Joins the closest server that already has players, or an empty one if none do, and opens its lobby."
                ]},
                { type: "h3", content: "Browse servers" },
                { type: "list", items: [
                    "Lists every server with its region, map, players, ping and version.",
                    "Password-protected servers ask for the password when you join."
                ]},
                { type: "h3", content: "Skirmish lobby" },
                { type: "list", items: [
                    "Two teams face each other. Fill open seats with players or with <b>Add AI</b>, and set each AI's <b>Difficulty</b> on its slot.",
                    "One player can watch from the <b>Spectator</b> seat.",
                    "The host can <b>Change map</b> and set <b>Lobby Options</b>, including <b>Tweaks</b>, presets and a <b>Match password</b>.",
                    "The host starts the match with <b>Start battle</b>."
                ]}
            ]
        }
    };

    $scope.setActiveGuide = function(bookId) {
        $scope.activeGuideId = bookId;
    };

    // Returns raw HTML string (Angular 1.1.x handles the injection via ng-bind-html-unsafe)
    $scope.getCurrentGuideHtml = function() {
        var data = ($scope.supportAurora() && auroraGuideOverrides[$scope.activeGuideId]) || structuredGuideDatabase[$scope.activeGuideId];
        if (!data) {
            return '<p>Content for this guide is currently under construction.</p>';
        }

        var html = '<h2 class="guide-page-header">' + data.title + '</h2>';

        function buildListHtml(items) {
            var out = '<ul>';
            for (var i = 0; i < items.length; i++) {
                var item = items[i];
                if (typeof item === 'string') {
                    out += '<li>' + item + '</li>';
                } else {
                    out += '<li>' + item.text;
                    if (item.subItems) {
                        out += buildListHtml(item.subItems);
                    }
                    if (item.image) {
                        out += '<br><img src="' + item.image.src + '" style="' + (item.image.style || '') + '" alt="' + item.image.alt + '">';
                    }
                    out += '</li>';
                }
            }
            out += '</ul>';
            return out;
        }

        for (var i = 0; i < data.blocks.length; i++) {
            var block = data.blocks[i];
            
            if (block.type === 'p') {
                html += '<p>' + block.content + '</p>';
            } else if (block.type === 'h3') {
                html += '<h3>' + block.content + '</h3>';
            } else if (block.type === 'image-wrap') {
                var style = block.style || 'width:100%; ';
                var wrapStyle = block.wrapStyle ? (' style="' + block.wrapStyle + '"') : '';
                var altText = block.alt ? (' alt="' + block.alt + '"') : '';
                html += '<div class="guide-image-wrap"' + wrapStyle + '><img src="' + block.src + '" style="' + style + '"' + altText + '></div>';
            } else if (block.type === 'image-panel') {
                var imagesHtml = '';
                if (typeof block.src === 'string') {
                    imagesHtml = '<img src="' + block.src + '" alt="' + (block.alt || '') + '">';
                } else if (Array.isArray(block.src)) {
                    for (var j = 0; j < block.src.length; j++) {
                        imagesHtml += '<img src="' + block.src[j] + '" alt="' + (block.alt || '') + '" style="margin-right: 5px;">';
                    }
                }
                html += '<div class="guide-image-panel">' +
                            '<div class="panel-label">' + block.label + '</div>' +
                            '<div class="panel-image">' + imagesHtml + '</div>' +
                        '</div>';
            } else if (block.type === 'list') {
                html += buildListHtml(block.items);
            } else if (block.type === 'table') {
                var tableStyle = block.tableStyle || 'width:100%; text-align:left; border-collapse: collapse; margin: 20px 0;';
                html += '<table style="' + tableStyle + '">';
                
                if (block.headers && !block.hideHeaders) {
                    html += '<tr style="border-bottom: 1px solid #333;">';
                    for (var h = 0; h < block.headers.length; h++) {
                        var head = block.headers[h];
                        if (typeof head === 'string') {
                            html += '<th style="padding: 10px; color: #fff;">' + head + '</th>';
                        } else {
                            var headStyle = head.style ? (' style="padding: 10px; color: #fff; ' + head.style + '"') : ' style="padding: 10px; color: #fff;"';
                            html += '<th' + headStyle + '>' + head.content + '</th>';
                        }
                    }
                    html += '</tr>';
                }
                
                for (var r = 0; r < block.rows.length; r++) {
                    var row = block.rows[r];
                    var trStyle = (r < block.rows.length - 1 && !block.tableStyle) ? ' style="border-bottom: 1px solid #333;"' : '';
                    if (row[0] && row[0].trStyle) {
                        trStyle = ' style="' + row[0].trStyle + '"';
                    }
                    html += '<tr' + trStyle + '>';
                    
                    for (var c = 0; c < row.length; c++) {
                        var cell = row[c];
                        var cellStyle = 'padding: 10px;';
                        if (cell.style) {
                            cellStyle = cell.style; // Allow complete style override
                        }
                        
                        html += '<td style="' + cellStyle + '">';
                        
                        if (typeof cell === 'string') {
                            html += cell;
                        } else if (cell.type === 'list') {
                            html += buildListHtml(cell.items);
                        } else if (cell.type === 'html') {
                            html += cell.content;
                        } else if (cell.content) {
                            html += cell.content;
                        }
                        
                        html += '</td>';
                    }
                    html += '</tr>';
                }
                html += '</table>';
            }
        }
        
        return $scope.supportAurora() ? $scope.auroraWording(html) : html;
    };

});
