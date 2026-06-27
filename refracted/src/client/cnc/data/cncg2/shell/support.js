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
            text: "Yes, you can find our beginner's guide and a basic tutorial for the EU under the \"Learn\" tab. We will be expanding our training and tutorial elements over the coming year.",
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

});