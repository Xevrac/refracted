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
        text: "Welcome to the Command & Conquer tutorial! This section will progressively expand with interactive training tools to help you improve your skills. To get hands-on training, launch the EU tutorial. We also recommend checking out the BEGINNER'S GUIDE (Learn section). Online beginner's guide",
        buttonText: "Launch EU Tutorial"
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
        { id: 'modes', title: "GAME MODES" }
    ];

    $scope.activeGuideId = 'ui'; // Default active book

    // Simulated payload of large HTML documents
    var rawGuideDatabase = {
        'ui': '<h2 class="guide-page-header">USER INTERFACE</h2><p>The User Interface is what you will use to Build and Command your Units and Structures.</p><div class="guide-image-wrap"><img src="view/image/debugPinkRect.png" style="width:100%; height:300px; border:1px solid #333;" alt="User Interface Map Overview"></div>',
        'basic': '<h2 class="guide-page-header">BASIC CONTROLS</h2><p>Moving units and issuing attack orders forms the foundation of all combat scenarios.</p><ul><li><strong>Left Click:</strong> Select unit or structure.</li><li><strong>Right Click:</strong> Issue move or attack order.</li></ul>',
        'adv': '<h2 class="guide-page-header">ADVANCED CONTROLS</h2><p>Mastering control groups and waypoints.</p>'
    };

    $scope.setActiveGuide = function(bookId) {
        $scope.activeGuideId = bookId;
    };

    // Returns raw HTML string (Angular 1.1.x handles the injection via ng-bind-html-unsafe)
    $scope.getCurrentGuideHtml = function() {
        var rawHtml = rawGuideDatabase[$scope.activeGuideId];
        if (!rawHtml) {
            rawHtml = '<p>Content for this guide is currently under construction.</p>';
        }
        return rawHtml;
    };

});