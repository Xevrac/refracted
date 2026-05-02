/**
 * Credits Controller
 * Supplies the structured data for the cascading typography in the credits modal.
 */
CCApp.controller('CreditsController', function($scope) {
    
    $scope.creditsData = [
        {
            category: "ART",
            sections: [
                {
                    subtitle: "[Los-Angeles-Team]",
                    entries: [
                        { role: "Art Director", name: "Chris Tamburrino" },
                        { role: "Development Director II", name: "Nicole West" }
                    ]
                },
                {
                    subtitle: "Animation",
                    entries: [
                        { role: "Lead Animator", name: "Umberto Bossi" },
                        { role: "Senior Animator", name: "Jane Doe" }
                    ]
                },
                {
                    subtitle: "Environment",
                    entries: [
                        { role: "Lead Environment Artist", name: "John Smith" }
                    ]
                }
            ]
        },
        {
            category: "DESIGN",
            sections: [
                {
                    subtitle: "[Core Gameplay]",
                    entries: [
                        { role: "Lead Designer", name: "Samuel Bass" }
                    ]
                }
            ]
        }
    ];

});