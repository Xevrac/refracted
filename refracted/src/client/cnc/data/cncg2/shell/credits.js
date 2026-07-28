/**
 * Credits Controller
 * Supplies the structured data for the cascading typography in the credits modal.
 */
CCApp.controller('CreditsController', function($scope) {
    
    $scope.creditsData = [
        {
            category: "",
            sections: [
                {
                    subtitle: "",
                    entries: [
                        { 
                            role: "Vice President and General Manager", 
                            names: [
                                "Jon Van Caneghem"
                            ] 
                        },
                        { 
                            role: "Chief Operating Officer", 
                            names: [
                                "Amy Small"
                            ] 
                        },
                        { 
                            role: "Director, Product Development", 
                            names: [
                                "Tim Morten"
                            ] 
                        },
                        { 
                            role: "Creative Director", 
                            names: [
                                "Bryan Farina"
                            ] 
                        }
                    ]
                }
            ]
        },
        {
            category: "DESIGN",
            sections: [
                {
                    subtitle: "",
                    entries: [
                        { 
                            role: "Lead Designer", 
                            names: [
                                "Samuel Bass"
                            ] 
                        },
                        { 
                            role: "Development Director", 
                            names: [
                                "Dan Badilla"
                            ] 
                        },
                        { 
                            role: "Senior Technical Designer", 
                            names: [
                                "Jeremy Townsend"
                            ] 
                        },
                        { 
                            role: "Designers", 
                            names: [
                                "Michael Ombao", 
                                "Jacqueline Kate Salsman"
                            ] 
                        },
                        { 
                            role: "Design Engineer", 
                            names: [
                                "Keith Yates"
                            ] 
                        },
                        { 
                            role: "Associate Producer", 
                            names: [
                                "Jon LeMaitre"
                            ] 
                        },
                        { 
                            role: "Additional Design", 
                            names: [
                                "Chris Rockoff", 
                                "Steve Copeland"
                            ] 
                        }
                    ]
                }
            ]
        },
        {
            category: "ENGINEERING",
            sections: [
                {
                    subtitle: "[Austin-Team]",
                    entries: [
                        { 
                            role: "Technical Director", 
                            names: [
                                "Jay Lee"
                            ] 
                        },
                        { 
                            role: "Development Director", 
                            names: [
                                "Jim Hudson"
                            ] 
                        },
                        { 
                            role: "Engineers", 
                            names: [
                                "Luke Bagwell", 
                                "Sean Boocock", 
                                "Ian Bullard", 
                                "Derek Hall", 
                                "Todd Hayes", 
                                "Anthony Maurice", 
                                "Eli Pulsifer", 
                                "Brandon Rowlett", 
                                "Michael Songy"
                            ] 
                        }
                    ]
                },
                {
                    subtitle: "[Los Angeles-Team]",
                    entries: [
                        { 
                            role: "LA Engineering Lead", 
                            names: [
                                "Andrew McDonald"
                            ] 
                        },
                        { 
                            role: "Engineers", 
                            names: [
                                "Steve Chow", 
                                "Scott Fallier", 
                                "Alexander Green", 
                                "Randy Stanton"
                            ] 
                        },
                        { 
                            role: "Additional Engineering", 
                            names: [
                                "Blair Hamilton", 
                                "Simon Myszko", 
                                "Bill Randolph"
                            ] 
                        }
                    ]
                }
            ]
        },
        {
            category: "ART",
            sections: [
                {
                    subtitle: "[Los Angeles-Team]",
                    entries: [
                        { 
                            role: "Art Director", 
                            names: [
                                "Chris Tamburrino"
                            ] 
                        },
                        { 
                            role: "Development Director II", 
                            names: [
                                "Nicole West"
                            ] 
                        }
                    ]
                },
                {
                    subtitle: "Animation",
                    entries: [
                        { 
                            role: "Lead Animator", 
                            names: [
                                "Umberto Bossi"
                            ] 
                        }
                    ]
                },
                {
                    subtitle: "Environment",
                    entries: [
                        { 
                            role: "Lead Environment Artist", 
                            names: [
                                "Robert Keenan"
                            ] 
                        },
                        { 
                            role: "Senior Environment Artist", 
                            names: [
                                "Craig Marschke"
                            ] 
                        }
                    ]
                },
                {
                    subtitle: "Modeling",
                    entries: [
                        { 
                            role: "Modeling Director", 
                            names: [
                                "Tse-cheng Lo"
                            ] 
                        },
                        { 
                            role: "Modeling Artist", 
                            names: [
                                "Rory McMahon"
                            ] 
                        }
                    ]
                },
                {
                    subtitle: "Technical Art",
                    entries: [
                        { 
                            role: "Technical Art Director", 
                            names: [
                                "Gary Snyder"
                            ] 
                        },
                        {
                            role: "Technical Artist",
                            names: [
                                "Kyle Nikolich",
                                "Janey Yang"
                            ]
                        }
                    ]
                },
                {
                    subtitle: "Visual Effects",
                    entries: [
                        { 
                            role: "Lead VFX Artist", 
                            names: [
                                "Aram Granger"
                            ] 
                        }
                    ]
                },
                {
                    subtitle: "[Shanghai-Team]",
                    entries: [
                        { 
                            role: "Associate Art Director", 
                            names: [
                                "Stacey Jamieson"
                            ] 
                        },
                        { 
                            role: "Development Manager", 
                            names: [
                                "Jianjun (Michael) Yang"
                            ] 
                        }
                    ]
                },
                {
                    subtitle: "Animation",
                    entries: [
                        { 
                            role: "Animation Lead", 
                            names: [
                                "Shane Hu"
                            ] 
                        },
                        { 
                            role: "Animator", 
                            names: [
                                "Vicky Ge", 
                                "Hunk Yin"
                            ] 
                        }
                    ]
                },
                {
                    subtitle: "Concept",
                    entries: [
                        { 
                            role: "Senior Concept Lead", 
                            names: [
                                "Hui (Max) Ling"
                            ] 
                        },
                        { 
                            role: "Concept Specialist", 
                            names: [
                                "Jinjie Ruan"
                            ] 
                        }
                    ]
                },
                {
                    subtitle: "Environment",
                    entries: [
                        { 
                            role: "Project Lead", 
                            names: [
                                "Joe Xu"
                            ] 
                        },
                        { 
                            role: "Environment Artists", 
                            names: [
                                "Selina Fu", 
                                "Quki Han", 
                                "Yibing Lu", 
                                "Hexiao Mei"
                            ] 
                        }
                    ]
                },
                {
                    subtitle: "Modeling",
                    entries: [
                        { 
                            role: "Modeling Lead", 
                            names: [
                                "Spark Chen"
                            ] 
                        },
                        { 
                            role: "Art Specialists", 
                            names: [
                                "Chao Wang", 
                                "Fu Yang"
                            ] 
                        },
                        { 
                            role: "Modeling Artist", 
                            names: [
                                "Tina Chen", 
                                "Dekui Jiang", 
                                "Hao Jiang", 
                                "Lincoln Lin", 
                                "Harry Luo", 
                                "Ding Ma", 
                                "Tao Ran", 
                                "Zihuan Su", 
                                "Dapeng Wang", 
                                "Wen Zhang"
                            ] 
                        }
                    ]
                },
                {
                    subtitle: "Technical Art",
                    entries: [
                        { 
                            role: "Technical Artists", 
                            names: [
                                "Jarod Feng", 
                                "Sheng Sam Yuan"
                            ] 
                        }
                    ]
                },
                {
                    subtitle: "Visual Effects",
                    isMissingData: true,
                    entries: [
                        { 
                            role: "VFX Artists", 
                            names: [
                                "Zhao Pu", 
                                "Ethan Huang"
                            ] 
                        },
                        { 
                            role: "Additional Art", 
                            names: [
                                "Stephen Camardella", 
                                "Leo Chen", 
                                "Harvey Han", 
                                "Terry Hess", 
                                "Brian Judhan", 
                                "Wang Kai", 
                                "Johnson Li", 
                                "Lucas Lu", 
                                "Valerie Nunez", 
                                "Zhi Qu", 
                                "Alan Xu", 
                                "Jean Xu", 
                                "Glen Yang", 
                                "Matthew York",
                                "Yuki Zeng"
                            ] 
                        }
                    ]
                },
            ]
        },
        {
            category: "FRONT END ENGINEERING",
            sections: [
                {
                    subtitle: "",
                    entries: [
                        { 
                            role: "Web Manager", 
                            names: [
                                "Alex Bayegi"
                            ] 
                        },
                        { 
                            role: "Development Manager", 
                            names: [
                                "Jason Savopolos"
                            ] 
                        },
                        { 
                            role: "Front End Engineers", 
                            names: [
                                "Mahesh Gupta",
                                "Cody Massin",
                                "Brian McCain",
                                "Jason Micklewright",
                                "Daniel Murker",
                                "Dirk Rhynsburger",
                                "Joseph Shunk"
                            ] 
                        }
                    ]
                },
            ]
        },
        {
            category: "ANALYTICS",
            sections: [
                {
                    subtitle: "",
                    entries: [
                        { 
                            role: "Business Intelligence Manager", 
                            names: [
                                "Shawn Seibert"
                            ] 
                        }
                    ]
                }
            ]
        },
        {
            category: "LIVE",
            sections: [
                {
                    subtitle: "",
                    entries: [
                        { 
                            role: "Live Producer", 
                            names: [
                                "Mike Godsey"
                            ] 
                        },
                        { 
                            role: "Technical Operations Engineer", 
                            names: [
                                "Ben Hines"
                            ] 
                        }
                    ]
                }
            ]
        },
        {
            category: "STUDIO SUPPORT",
            sections: [
                {
                    subtitle: "",
                    entries: [
                        { 
                            role: "QA Testers", 
                            names: [
                                "Ethan Jeansonne",
                                "Sean Jeansonne",
                                "Kalin Jock",
                                "Amber Jones",
                                "Thomas Leger",
                                "Christopher Liaw",
                                "Mario Martinez",
                                "Matthew McClelland",
                                "Jake Meyers",
                                "Calvin Montgomery",
                                "Tyler Ono",
                                "Alex Roger",
                                "David Schweinefus",
                                "Steven Craig Shank",
                                "Tony Shorts",
                                "Delvin Spruel",
                                "Anthony Urrutia",
                                "Joseph Zachary"
                            ] 
                        },
                        { 
                            role: "Software Quality Engineer", 
                            names: [
                                "Narendra Umate"
                            ] 
                        }
                    ]
                },
                {
                    subtitle: "[Redwood Shores Team]",
                    entries: [
                        { 
                            role: "QA Compliance Manager", 
                            names: [
                                "Brian Yip"
                            ] 
                        },
                        { 
                            role: "QA Compliance Project Lead", 
                            names: [
                                "Seferino Gallardo"
                            ] 
                        },
                        { 
                            role: "QA Compliance Analyst", 
                            names: [
                                "Manny Coronado",
                                "Manny Grimaldo",
                                "Jeremy Hymel",
                                "Roman Janczak",
                                "Mario Martinez",
                                "Chris Mintzias",
                                "Ryan Sandberg",
                                "Jason Savopolos",
                                "Nathan Verbois",
                                "Keith Yates"
                            ] 
                        }
                    ]
                }
            ]
        },
        {
            category: "LOCALIZATION",
            sections: [
                {
                    subtitle: "[Guildford Team]",
                    entries: [
                        {
                            role: "International Project Manager",
                            names: [
                                "Sarah Turpin"
                            ]
                        },
                        {
                            role: "",
                            names: [
                                "Juan Arranz",
                                "Anastasiya Koroleva",
                                "Patrik Schweigl"
                            ]
                        },
                        {
                            role: "Engineering Project Lead",
                            names: [
                                "Ruben Martin Rico"
                            ]
                        },
                        {
                            role: "Software Engineer Project Manager",
                            names: [
                                "Javier Carazo Infestas"
                            ]
                        },
                        {
                            role: "Localization Software Engineer",
                            names: [
                                "Iker Aneiros"
                            ]
                        }
                    ]
                },
                {
                    subtitle: "[Cologne Team]",
                    entries: [
                        {
                            role: "Asset Localization",
                            names: [
                                "Marcel Elsner"
                            ]
                        }
                    ]
                }
            ]
        },
        {
            category: "MARKETING",
            sections: [
                {
                    isMissingData: true,
                    entries: []
                },
                {
                    subtitle: "",
                    entries: [
                        { 
                            role: "Social, Community, Online Marketing Manager", 
                            names: [
                                "Eric Krause"
                            ] 
                        },
                        { 
                            role: "Associate Product Marketing Manager", 
                            names: [
                                "Cameron Turner"
                            ] 
                        },
                        { 
                            role: "Cast", 
                            names: [
                                "Jim Ward",
                                "Ismail Bashey",
                                "Paul Nakauchi",
                                "Assaf Cohen",
                                "Matt Yang King",
                                "Nick Jameson",
                                "Sam Kalidi",
                                "Carlos Alazraqui",
                                "Crispin Freeman",
                                "Cas Anvar",
                                "Robin Atkin Downes",
                                "Yuri Lowenthal",
                                "Ronobir Lahiri",
                                "Sunil Malhotra",
                                "Edita Brychta",
                                "JB Blanc",
                                "Brian Tochi",
                                "Tohoru Masamune"
                            ] 
                        },
                        {
                            role: "External Casting, Audio Direction and Recording",
                            names: [
                                "Voice Works Productions",
                                "Douglass Carrigan",
                                "Jamie Siedow"
                            ]
                        },
                        {
                            role: "External Concept Art",
                            names: [
                                "Concept Art House",
                                "West Studios",
                                "Raymond Swanland"
                            ]
                        }
                    ]
                }
            ]
        },
        {
            category: "DICE",
            sections: [
                {
                    subtitle: "",
                    entries: [
                        {
                            role: "Frostbite Development Support",
                            names: [
                                "Joakim Lindqvist"
                            ]
                        }
                    ]
                }
            ]
        },
        {
            category: "EA DEVELOPER RELATIONS",
            sections: [
                {
                    subtitle: "",
                    entries: [
                        {
                            role: "",
                            names: [
                                "Adam Butterfoss",
                                "Ashley Bennett"
                            ]
                        }
                    ]
                }
            ]
        },
        {
            category: "EA DIGITAL PLATFORM INFRASTRUCTURE AND OPERATIONS",
            sections: [
                {
                    subtitle: "",
                    entries: [
                        {
                            role: "",
                            names: [
                                "Daniel W. Sheppard",
                                "Kirby Burchill",
                                "Matt Wygent",
                                "Dan Windrem",
                                "Kalyan Deka",
                                "Kurt Oehlschlaeger",
                                "Robert Lang",
                                "Guillermo Navarro",
                                "Sinclair Temple"
                            ]
                        }
                    ]
                }
            ]
        },
        {
            category: "EA NEW PRODUCT LAUNCH",
            sections: [
                {
                    subtitle: "",
                    entries: [
                        {
                            role: "",
                            names: [
                                "Thilo W. Huebner"
                            ]
                        }
                    ]
                }
            ]
        },
        {
            category: "EA MARKETING AND PUBLIC RELATIONS",
            sections: [
                {
                    subtitle: "",
                    entries: [
                        {
                            role: "Vice President Worldwide Marketing",
                            names: [
                                "Rich Williams"
                            ]
                        },
                        {
                            role: "Senior Director Worldwide Marketing",
                            names: [
                                "Craig Owens"
                            ]
                        },
                        {
                            role: "Public Relations Manager",
                            names: [
                                "Melissa Ojeda"
                            ]
                        },
                        {
                            role: "Public Relations Coordinator",
                            names: [
                                "Kristiana Suess"
                            ]
                        },
                        {
                            role: "Senior Coordinator Public Relations",
                            names: [
                                "Stephanie Driscoll"
                            ]
                        },
                        {
                            role: "Director of Brand Strategy",
                            names: [
                                "Kristen Salvatore"
                            ]
                        },
                        {
                            role: "Asia Pacific Marketing Manager",
                            names: [
                                "Craig Auld"
                            ]
                        },
                        {
                            role: "Senior Video Project Manager",
                            names: [
                                "Neal Upadhya"
                            ]
                        },
                        {
                            role: "Marketing Video Editors",
                            names: [
                                "Chase Boyajian",
                                "Tanner Boyajian"
                            ]
                        },
                        {
                            role: "Marketing Sound Designer",
                            names: [
                                "Charley Stauber"
                            ]
                        },
                        {
                            role: "Marketing Coordinator",
                            names: [
                                "Jonathan Judelson"
                            ]
                        }
                    ]
                }
            ]
        },
        {
            category: "EA LEGAL",
            sections: [
                {
                    subtitle: "",
                    entries: [
                        {
                            role: "",
                            names: [
                                "Sue Garfield",
                                "Katie Huang",
                                "Stu Eaton",
                                "Thao Tran",
                                "Amy Saechao",
                                "Joe Cademartori"
                            ]
                        }
                    ]
                }
            ]
        },
        {
            category: "IS&T",
            sections: [
                {
                    subtitle: "",
                    entries: [
                        {
                            role: "",
                            names: [
                                "Joe Aguilar",
                                "Steve Arnold",
                                "Doug Brayfield",
                                "Jimmie Harlow",
                                "Tu Holmes",
                                "Isaac Lee",
                                "Nick Lirio",
                                "Michael Love",
                                "Roni Papouban",
                                "Ray Robinson",
                                "Manny Sherbany",
                                "Louie Soriano",
                                "Angel Zavala"
                            ]
                        }
                    ]
                }
            ]
        },
        {
            category: "PRODUCTION BABIES",
            sections: [
                {
                    subtitle: "",
                    entries: [
                        {
                            role: "",
                            names: [
                                "Kira Badilla",
                                "JiaMing Chen",
                                "Dylan McCain",
                                "Tristan Green",
                                "Hayley Hall",
                                "Dylan McDonald",
                                "TianXi Wang"
                            ]
                        }
                    ]
                }
            ]
        },
        {
            category: "LEGAL NOTICES",
            sections: [
                {
                    subtitle: "",
                    entries: [
                        {
                            role: "",
                            names: [
                                "Command & Conquer uses Havok\u00AE.",
                                "\u00A9Copyright 1999-2013 Havok.com Inc. (and",
                                "its Licensors). All Rights Reserved. See",
                                "www.havok.com for details.",
                                "Copyright in Enlighten is owned by or licensed",
                                "to Geomerics Limited. All rights reserved.",
                                "Enlighten is a trademark or registered",
                                "trademark of Geomerics Limited or its",
                                "affiliates.",
                                "Portions of this software utilize",
                                "SpeedTree\u00AERT technology (\u00A92006",
                                "Interactive Data Visualization, Inc.).",
                                "SpeedTree\u00AE is a registered trademark of",
                                "Interactive Data Visualization, Inc. All rights",
                                "reserved.",
                                "This product may include in-game",
                                "sponsorships or product placements.",
                                "Tested at the EA North American Test",
                                "Center, a facility developed with the",
                                "assistance of the Louisiana Economic",
                                "Development's Office of Entertainment",
                                "Industry Development."
                            ]
                        }
                    ]
                }
            ]
        }
    ];

});