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
            question: "Is there a \"getting started\" guide?",
            text: "Yes! Please visit the LEARN tab or check our official forums for comprehensive guides and community tutorials to help you get onto the battlefield quickly."
        }
    ];

    $scope.troubleshootList = [
        {
            question: "My game is crashing on startup.",
            text: "Please ensure your graphic drivers are fully updated. If the issue persists, try running the 'Repair Install' option from the launcher."
        },
        {
            question: "I cannot connect to a multiplayer match.",
            text: "Check your firewall settings to ensure Command & Conquer is allowed through. Also, verify that the server status indicator in the bottom right corner is green."
        }
    ];

});