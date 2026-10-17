/**
 * Support Controller
 * Handles the text-heavy FAQ and Troubleshooting data loops.
 */
CCApp.controller('SupportController', function($scope) {
    
    $scope.faqList = [
        {
            question: "Is Command & Conquer finished?",
            text: "Command & Conquer is currently in Alpha, which means that it is in a playable, but unfinished, state. Many elements, from graphics to sound to gameplay balance, are works-in-progress and will gain significant improvement and polish over the coming months. Your feedback is incredibly valuable in helping the team determine which elements to prioritize and which directions to take, so please share your opinions."
        },
        {
            question: "Who do I play as in Command & Conquer?",
            text: "You can play as one of three Factions \u2013",
            bullets: [
                "The Asia-Pacific Alliance (APA), a conglomerate of allied Asian countries. Play Style - emphasis on hordes of low-level Units, back by individual high-level Units.",
                "The European Union (EU), a high-tech, centralized single-state entity comprising all of Western and Northern Europe. Play Style - classic Command & Conquer, designed to be pick-up-and-play.",
                "The Global Liberation Army (GLA), a stateless terrorist organization, encompassing insurgent groups from all corners of the globe. Play Style - finesse-oriented, with stealth, sabotage and self-destruction."
            ]
        },
        {
            question: "Is there a guide for beginners?",
            text: "Yes, there is a beginner's guide and a basic tutorial for the EU. We will be expanding our training and tutorial elements over the coming year.",
            actionText: "LEARN",
            actionTab: "LEARN",
            actionSubTab: "PLAY TUTORIAL"
        },
        {
            question: "How can I provide feedback to the development team?",
            text: "We greatly appreciate your feedback, so please post your thoughts in our feedback forum."
        },
        {
            question: "Why is the game called Command & Conquer, not Generals 2?",
            text: "Command & Conquer is not a single game, but a live service that will be expanded with new content in the coming years. Although we are starting in the Generals universe, we do not want to limit ourselves to it ... so the game is just the beginning."
        },
        {
            question: "Is an internet connection required to play?",
            text: "Yes, Command & Conquer uses client-server technology. This means that the gameplay logic resides on remote servers that our players can connect to. This allows us to better optimize game balance, better manage lag (i.e. the shared lag of earlier C&C titles is a thing of the past) and better prevent cheating. There are also many other advantages over previous Command & Conquer titles."
        },
        {
            question: "How will the game be expanded?",
            text: "Command & Conquer is designed as a live service that will grow and evolve over the next few years. We plan to add new units, structures, abilities and even factions, in addition to new maps and game modes."
        },
        {
            question: "What about the costs?",
            text: "At the moment, Command & Conquer allows players to purchase Generals and abilities using our in-game currency (CP). We are currently adding more aspects to the in-game store and will be working closely with the community team regarding new items and payment models."
        },
        {
            question: "Will maps cost anything?",
            text: "No, all maps will be provided for free so as not to divide our player base."
        },
        {
            question: "What is the Premium Membership?",
            text: "The Premium Membership grants access to a variety of tools and configuration options that allow players to customize Command & Conquer to their individual tastes."
        },
        {
            question: "What are the benefits of a Premium Membership?",
            text: "Players with a Premium Membership receive significantly more CP and XP in every game. This allows them to level up faster and receive more upgrades to optimize their army for their preferred strategies. In addition, Premium Members have the ability to create custom games with unique rules, to which they can invite friends and foes. If you want to use matchmaking instead, as a Premium Member you can define various filters to search for very specific games. However, these are just a few of the benefits that a Premium Membership brings. Look out for future announcements, as we will reveal more information before the game's release."
        },
        {
            question: "How do I become a Premium Member?",
            text: "During the Alpha test phase, all players automatically receive a Premium Membership."
        },
        {
            question: "A certain unit/ability/player power is more powerful than expected. Do you plan to fix this?",
            text: "We are constantly reviewing our data and player feedback to find out how we can further improve the gameplay. To ensure a fair and entertaining player experience, we will therefore adjust the game balance regularly."
        },
        {
            question: "Is unit movement via left-click supported? What about a side menu?",
            text: "In this early stage of development, Command & Conquer only supports unit movement via right-click and a menu bar at the bottom of the screen. However, as development progresses, we will evaluate additional interface and menu configurations that allow players to customize the player experience to their individual needs."
        },
        {
            question: "Will there be a replay feature?",
            text: "Although Command & Conquer currently does not have a replay feature, we would like to implement this into the game."
        },
        {
            question: "Will there be support for mods and custom maps?",
            text: "While we know that custom modifications and maps are extremely important to our community, we are unable to focus on these aspects at this time."
        },
        {
            question: "Will there be more single-player content in the future?",
            text: "In the Alpha phase, we are focusing on the multiplayer experience. As a live service, however, we can implement a wide variety of features over time - including comprehensive single-player and/or co-op campaigns."
        },
        {
            question: "How do I play with my friend?",
            text: "To ensure short wait times and to provide the Command & Conquer team with data on the available game modes, party creation has been temporarily disabled. The ability to invite friends to games will be re-enabled at a later time, however."
        },
        {
            question: "Can I deselect units?",
            text: "In the Options menu, you have the ability to deselect units. However, this is disabled by default. Thanks again for playing!"
        }
    ];

    $scope.troubleshootList = [
        {
            question: "Can't find your answer here?",
            text: "If you didn't find the answer you where looking, try our Answer HQ!",
            actionText: "HELP CENTER | ORIGIN FAQ",
            image: "images/support/answers_logo.jpg"
        },
        {
            question: "Do I need Origin?",
            text: "Yes, Command & Conquer requires Origin to play."
        },
        {
            question: "Where can I get Origin?",
            text: "Go to Origin.com and you will find where you can download Origin to your PC."
        },
        {
            question: "How do I adjust my graphics settings?",
            text: "From the options menu, clicking on the Graphics Options button will open up the graphics options. Currently Command & Conquer only supports the modification of the overall resolution. Advanced graphic options will be made available at a later date."
        },
        {
            question: "How do I add friends?",
            text: "Friends are added to Command & Conquer's in-game friends list by automatically syncing your friends list through Origin. If you and another player are already Origin Friends, then that friend will automatically show up in the friends feed in-game. If you want to add a player as a friend in Command & Conquer you can access Origin In-Game to add them. Access Origin In-Game by Pressing Shift + F1 to access the Origin in-game overlay. In the overlay, select the Friends icon from the bottom toolbar in order to access your Origin Friends list. Invite the player you wish to friend through a request from the Origin Friends list. Once the player has accepted your friend request they will appear in the friends feed in-game."
        },
        {
            question: "How do I access the options from the main menu?",
            text: "When in the main menu, click on the options button in the upper right hand corner of the main menu. This will open up the options menu."
        },
        {
            question: "Can a friend and I team up and play random other players in 2v2 Deathmatch?",
            text: "At this time Command & Conquer does not support players creating groups to search for match in opponents."
        },
        {
            question: "How do I select a General?",
            text: "Generals are selectable by clicking on the button next to the general image in the top left corner of the main menu. Any generals that you purchase with CP will be added to this list, your currently selected general always appears in the box next to the button to select a general."
        },
        {
            question: "How do I access the tutorial?",
            text: "In order to access the tutorial, select the Learn button on the Main Menu. This will open up the Beginner's Guide for the user. Next to the header of the Beginner's Guide, a button is present called \"Play Tutorial\". Pressing this button will launch you into the tutorial."
        },
        {
            question: "How do I find a match?",
            text: "To find a multiplayer match in Command & Conquer, press the Play button from the main menu. If you wish to change your game mode click on the arrow to the right of the play button to select a game mode."
        },
        {
            question: "How do I stop searching for a match?",
            text: "Once you click play to start searching for a match the play button changes to a cancel button, simply click on the cancel button to cancel searching for other players."
        },
        {
            question: "How do I exit a game in progress?",
            text: "Once in a game, pressing the Esc will open up the options menu. If you select the Exit Game button, you will completely exit out of Command & Conquer. If you wish to return to the main menu to play a different game mode or find a different opponent, use the surrender system. Both surrendering and exiting Command & Conquer mid-in-game counts as a loss."
        },
        {
            question: "How do I pick my faction?",
            text: "Factions are determined by choosing a general from the general selector (top left of the main menu) that belongs to that faction, for example if you select a GLA general then you will be playing as the GLA faction."
        },
        {
            question: "How do I access Command & Conquer?",
            text: "In order to access and play Command & Conquer, you must redeem the product code within Origin to unlock the title in your Origin account. Once the title has been unlocked and linked to your Origin account, the game will appear within your games library in Origin. From the games library you will be able to download, install and play Command & Conquer."
        },
        {
            question: "I am having issues with Windows 8.",
            text: "Currently Command & Conquer does not support Windows 8. Please contact our support site (url) if you encounter any issues while using Windows 8 and we will evaluate the task at our studio."
        },
        {
            question: "My game froze?",
            text: "If the game freezes, close out of the program using the Windows Task Manager and restart the program. Please be aware that Command & Conquer is still a work in progress and freezes may occur on a regular basis. If restarting the program does not allow you to play the game, please contact support for further assistance."
        },
        {
            question: "I receive a server error whenever I search for a match.",
            text: "If there is a server error while searching for a match, close and restart the game. If that does not work, contact support."
        },
        {
            question: "A crash happens whenever I launch into a game.",
            text: "Ensure that you have the most recent version of Command & Conquer installed. Crashes can occur if two players on different versions attempt to connect to one another. In order to ensure that you have the latest version of Command & Conquer, use the check for updates option within the Origin games library."
        },
        {
            question: "Getting an error getting into the main menu (signing in).",
            text: "If there is an error getting into the main menu, close and restart the game. If that does not work, contact support."
        },
        {
            question: "Can I play single player?",
            text: "Single player skirmish against the AI is available. In order to play skirmish against the AI press the drop down menu under the Play button to bring up the game options. Select Practice PvE game mode and click Play, this will launch you into a game against a random AI."
        },
        {
            question: "Is there a campaign?",
            text: "Currently there is no campaign option available for the release of Command & Conquer. Command & Conquer is a product that will continuously evolve and we will constantly be adding new features and game modes into the mix the future."
        },
        {
            question: "I don't see friends online.",
            text: "If you do not see your friends online, check your internet connection to ensure that you are still online. If you are still online restart Command & Conquer. If friends are still not appearing after restarting the game please contact support for further assistance."
        },
        {
            question: "Why can't I select maps during matchmaking?",
            text: "Currently our matchmaking process randomly selects a map based on your selected game mode."
        },
        {
            question: "Why is it only 1v1 in skirmish?",
            text: "Skirmish is currently set as 1v1 at this time to provide the best experience in terms of challenge at the moment. The AI is currently optimized for 1 on 1 player interaction and provides the best challenge and most polished experience. We are working hard to provide additional skirmish variants in future versions of Command & Conquer."
        },
        {
            question: "How do I chat with friends?",
            text: "In order to chat with your friends from the Main menu, select the Friends Tab in the chat area. Right Click on an online friend to bring up the option to invite the friend to a private chat. After the invite has been sent, click on the Chat Tab in the chat area and select the friend's name to join a private room to chat with them."
        },
        {
            question: "How do I unlock achievements?",
            text: "Achievements can be unlocked by performing specific in game actions. A list of what achievements are available and unlocked can be found by opening the Profile area in the main menu then navigating to the Achievements Tab."
        },
        {
            question: "How do I view my post game statistics?",
            text: "From the Main Menu, select the Profile Tab. This will open up lists that displays a variety of personal statistics. Tabs will be present for in-depth statistics, leaderboards, match history and achievements."
        },
        {
            question: "How do I get in contact with support?",
            text: "For further support for issues, please use the Command & Conquer forums in order to reach a member of our community team."
        },
        {
            question: "How do I exit the game from the main menu?",
            text: "In order to exit the game from the main menu, select the options button in the upper right-hand corner of the Main Menu screen, then select the Quit Game button. This will close Command & Conquer and return you to the Desktop."
        }
    ];

    // Aurora view: the same topics, rewritten for Aurora. The lists above stay as the Original view.
    var auroraContent = {
        FAQ: [
            { id: 'game', title: "THE GAME", entries: [
                { q: "What is Aurora?", a: "Aurora brings back <b>Command & Conquer (2013)</b>, the Generals 2 Alpha that was cancelled before its online services launched. It runs on <b>Refracted</b>, which stands in for those online services, and <b>Prism</b>, which patches the game client and the dedicated servers." },
                { q: "Is Aurora finished?", a: "No. Aurora is a work in progress, and the game itself was still in Alpha when development stopped. Features come back build by build, and anything you see may change. Your build numbers are shown at the top left of the main menu." },
                { q: "Does Aurora cost anything?", a: "No. Aurora is a free hobby project. There is no store, currency or membership, and every map is available to everyone. If you would like to help the project, use <b>Support us</b> in the bottom bar." },
                { q: "Is an internet connection required to play?", a: "Yes. Matches run on dedicated servers: the game logic lives on the server and every player connects to it. This keeps lag with the player who has it (the shared lag of earlier C&C titles is a thing of the past) and makes cheating harder." },
                { q: "Will Aurora be expanded?", a: "The goal is to bring back everything the 2013 Alpha had. New features are announced in the <b>News</b> panel on the Play page as they arrive." },
                { q: "Will there be single-player content?", a: "You can already play against AI in a skirmish lobby. There is no campaign; the Alpha never had one. A tutorial mission is coming soon under <b>Support > Play tutorial</b>." },
                { q: "Why was the game called Command & Conquer, not Generals 2?", a: "EA planned Command & Conquer as a live service rather than a single game. It started in the Generals universe, but the name left room to grow beyond it." }
            ]},
            { id: 'factions', title: "FACTIONS", table: 'factions',
              intro: "You can play as one of three Factions.",
              rows: [
                ["The Asia-Pacific Alliance (APA)", "A conglomerate of allied Asian countries.", "Emphasis on hordes of low-level Units, backed by individual high-level Units."],
                ["The European Union (EU)", "A high-tech, centralized single-state entity comprising all of Western and Northern Europe.", "Classic Command & Conquer, designed to be pick-up-and-play."],
                ["The Global Liberation Army (GLA)", "A stateless terrorist organization, encompassing insurgent groups from all corners of the globe.", "Finesse-oriented, with stealth, sabotage and self-destruction."]
              ]},
            { id: 'start', title: "GETTING STARTED", entries: [
                { q: "Is there a guide for beginners?", a: "Yes. <b>Support > Guides</b> covers the interface, controls, resources, structures, production and more. A tutorial mission for the EU is coming soon.", action: { text: "GUIDES", tab: 'SUPPORT', subTab: 'GUIDES' } },
                { q: "How do I play with a friend?", a: "Both of you open <b>Browse servers</b> on the Play page and join the same server. The lobby host can set a <b>Match password</b> to keep the seats for you. Invites are coming soon." },
                { q: "Can I deselect units?", a: "Yes. Turn on <b>Options > Gameplay > Allow Deselect</b>." },
                { q: "Is unit movement via left-click supported?", a: "Not at the moment. Units move with right-click, and the command bar sits at the bottom of the screen." },
                { q: "How can I give feedback?", a: "Pass it on to the Aurora team (Xevrac and Nemo). Include the build numbers from the top left of the main menu (<b>RFR</b>, <b>PRISM</b> and <b>CNC</b>) so we know which build you were playing." }
            ]},
            { id: 'features', title: "BALANCE & FEATURES", entries: [
                { q: "A unit, ability or player power seems too strong. Will it be fixed?", a: "Aurora uses the game's own 2013 balance data rather than making up new values, so units should play as they did in the Alpha. If something looks wrong, tell us: it may be a bug in how a rule is applied." },
                { q: "Will there be a replay feature?", a: "There is no replay feature at the moment." },
                { q: "Will there be support for mods and custom maps?", a: "Not at the moment. Lobby hosts can change how a match plays under <b>Lobby Options > Tweaks</b> and save their settings as a preset." }
            ]}
        ],
        TROUBLESHOOT: [
            { id: 'problems', title: "COMMON PROBLEMS", table: 'problems', rows: [
                ["The game froze.", "Close it with Windows Task Manager and start it again from the <b>Refracted launcher</b>. Aurora is a work in progress, so freezes can happen. If it keeps happening, let the Aurora team know."],
                ["Find a match shows an error.", "Click <b>OK</b> and try again, or pick a server yourself with <b>Browse servers</b>. If it keeps failing, restart the game."],
                ["The game crashes when a match starts.", "Players on different versions can crash when they connect to each other. Make sure the launcher has updated Prism, and check the <b>Version</b> column in <b>Browse servers</b> before you join."],
                ["An error appears while signing in.", "Close the game and start it again from the <b>Refracted launcher</b>. If that does not help, contact the Aurora team."],
                ["Units react late or the game stutters.", "Look for the warning icons at the top right of the screen. <b>Support > Guides > User Interface</b> explains each one and what you can do about it."],
                ["No servers are listed.", "Click <b>Refresh</b> in <b>Browse servers</b>. The <b>servers online</b> count at the bottom right of the main menu shows how many are running."]
            ]},
            { id: 'account', title: "ACCOUNT & UPDATES", entries: [
                { q: "What do I need to play?", a: "Aurora requires Refracted. Start the game from the <b>Refracted launcher</b>: it signs you in and installs Prism for you." },
                { q: "How do I keep Aurora up to date?", a: "The Refracted launcher checks for Prism updates when you start it and installs them before the game launches." }
            ]},
            { id: 'menus', title: "MENUS & SETTINGS", entries: [
                { q: "How do I open the options?", a: "Click <b>Options</b> in the bottom bar of the main menu. In a match, press <b>Esc</b> and choose <b>Options</b>." },
                { q: "How do I adjust my graphics settings?", a: "Open <b>Options > Graphics</b>. You can set the fullscreen and windowed resolutions, switch to windowed mode and change the graphics quality." },
                { q: "How do I access the tutorial?", a: "Open <b>Support > Play tutorial</b>. The EU tutorial mission is coming soon; until then, <b>Support > Guides</b> covers the basics." },
                { q: "How do I exit the game from the main menu?", a: "Click <b>Quit</b> in the bottom bar." }
            ]},
            { id: 'generals', title: "GENERALS & FACTIONS", entries: [
                { q: "How do I select a General?", a: "Pick your General on your slot in the lobby. To start every new lobby with the same one, set <b>Preferred Faction</b> and <b>Preferred General</b> under <b>Options > Gameplay > Lobby Defaults</b>. The main menu shows the General you played last." },
                { q: "How do I pick my faction?", a: "Choose your <b>Faction</b> on your slot in the lobby, next to your General." },
                { q: "Can I customise my Generals?", a: "Not yet. Customising Generals and loadouts is coming soon under <b>Profile > Generals</b>." }
            ]},
            { id: 'matchmaking', title: "MATCHMAKING", entries: [
                { q: "How do I find a match?", a: "Click <b>Find a match</b> on the Play page. Aurora joins the closest server that already has players, or an empty one if none do, and opens its lobby." },
                { q: "How do I pick a server myself?", a: "Click <b>Browse servers</b> on the Play page. Each server is listed with its region, map, players, ping and version." },
                { q: "How do I leave a lobby?", a: "Click <b>Leave</b> in the lobby to return to the main menu." },
                { q: "Can I choose the map?", a: "Find a match takes the map the server is on. To choose, check the <b>Map</b> column in <b>Browse servers</b>, or, as lobby host, use <b>Change map</b>." },
                { q: "Can a friend and I team up against other players?", a: "Yes. The lobby has two teams: join the same team and fill the other seats with players or AI." },
                { q: "How do I exit a game in progress?", a: "Press <b>Esc</b> to open the game menu. <b>Surrender</b> ends the match as a loss, and <b>Quit game</b> closes the game." }
            ]},
            { id: 'skirmish', title: "SKIRMISH", entries: [
                { q: "Can I play against AI?", a: "Yes. In a skirmish lobby, click <b>Add AI</b> under a team and set its <b>Difficulty</b> on its slot. AI can go on either team, so you can play alongside it or against it." },
                { q: "Is there a campaign?", a: "No. There is no campaign; the 2013 Alpha did not have one." }
            ]},
            { id: 'friends', title: "FRIENDS & CHAT", entries: [
                { q: "How do I add friends?", a: "Aurora does not have a friends list yet. To play together, join the same server through <b>Browse servers</b>. Invites are coming soon." },
                { q: "How do I chat with other players?", a: "Use the lobby chat (<b>Message the lobby</b>) to talk to everyone in your lobby." }
            ]},
            { id: 'profile', title: "PROFILE", entries: [
                { q: "How do I unlock achievements?", a: "Achievements unlock by doing specific things in a match. Open <b>Profile > Achievements</b> from the Play page to see which ones are available and which you have." },
                { q: "How do I view my statistics?", a: "Open <b>Profile</b> from the Play page. It has tabs for your Overview, Statistics, Match history and Leaderboards. Records are kept per realm, so each realm has its own wins and losses." }
            ]},
            { id: 'help', title: "GETTING HELP", entries: [
                { q: "Can't find your answer here?", a: "Check the <b>News</b> panel on the Play page for known issues and new builds, then contact the Aurora team (Xevrac and Nemo). Include your build numbers from the top left of the main menu (<b>RFR</b>, <b>PRISM</b> and <b>CNC</b>)." }
            ]}
        ]
    };

    var activeSections = { FAQ: 'game', TROUBLESHOOT: 'problems' };

    $scope.currentSections = function () {
        return auroraContent[$scope.activeSubTab] || [];
    };

    $scope.activeSection = function () {
        return activeSections[$scope.activeSubTab];
    };

    $scope.setActiveSection = function (id) {
        activeSections[$scope.activeSubTab] = id;
    };

    $scope.onArticleClick = function ($event) {
        var el = $event.target;
        if (!el || !el.getAttribute || !el.getAttribute('data-tab')) {
            return;
        }
        $scope.setTab(el.getAttribute('data-tab'));
        $scope.setSubTab(el.getAttribute('data-subtab'));
    };

    function entryHtml(entry) {
        var html = '<h3>' + entry.q + '</h3><p>' + entry.a;
        if (entry.action) {
            html += ' <a class="support-action" data-tab="' + entry.action.tab + '" data-subtab="' + entry.action.subTab + '">' + entry.action.text + '</a>';
        }
        return html + '</p>';
    }

    function tableHtml(headers, widths, rows) {
        var html = '<table><tr>';
        for (var h = 0; h < headers.length; h++) {
            html += '<th style="padding: 10px;' + (widths[h] ? ' width: ' + widths[h] + ';' : '') + '">' + headers[h] + '</th>';
        }
        html += '</tr>';
        for (var r = 0; r < rows.length; r++) {
            html += '<tr>';
            for (var c = 0; c < rows[r].length; c++) {
                html += '<td style="padding: 10px;">' + (c === 0 ? '<b>' + rows[r][c] + '</b>' : rows[r][c]) + '</td>';
            }
            html += '</tr>';
        }
        return html + '</table>';
    }

    $scope.getCurrentSectionHtml = function () {
        var sections = auroraContent[$scope.activeSubTab];
        if (!sections) {
            return '';
        }
        var section = sections[0];
        for (var s = 0; s < sections.length; s++) {
            if (sections[s].id === activeSections[$scope.activeSubTab]) {
                section = sections[s];
            }
        }

        var html = '<h2 class="guide-page-header">' + section.title + '</h2>';
        if (section.intro) {
            html += '<p>' + section.intro + '</p>';
        }
        if (section.table === 'factions') {
            html += tableHtml(["Faction", "Who they are", "Play style"], ['220px'], section.rows);
        } else if (section.table === 'problems') {
            html += tableHtml(["Problem", "What to do"], ['320px'], section.rows);
        } else {
            for (var i = 0; i < section.entries.length; i++) {
                html += entryHtml(section.entries[i]);
            }
        }
        return html;
    };

});