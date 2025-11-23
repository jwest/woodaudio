/*
 * Parametryczny moduł Keycapa dla przełącznika typu MX.
 */

// Ustawienie domyślnej rozdzielczości
$fn = 50;

/**
 * Generuje pojedynczy keycap dla przełącznika MX.
 *
 * @param u_size Całkowita szerokość i głębokość keycapa (np. 18mm dla 1U)
 * @param height Całkowita wysokość keycapa
 * @param top_offset Wartość zmniejszająca górną powierzchnię (tworzy skośne boki)
 * @param corner_radius Promień zaokrąglenia narożników górnej powierzchni
 * @param stem_height Wysokość wewnętrznego trzpienia MX
 * @param stem_tol Tolerancja dla krzyżowego trzpienia MX (kluczowy wymiar dla pasowania)
 */
module mx_keycap(u_size = 23.1, height = 8, top_offset = 1.5, corner_radius = 1.5, stem_height = 4.5, stem_tol = 0.1) {
    
    // --- Definicje wymiarów MX Stem (Standardowe) ---
    // Krzyżowy trzpień MX ma boki ok. 3.9mm, a 'puste' przestrzenie ok. 1.2mm
    // Wymaga to precyzyjnego modelowania przestrzeni negatywnej.
    
    stem_arm_width = 1.2 + stem_tol;  // Szerokość "ramion" krzyża (przestrzeń negatywna)
    stem_total_length = 4.0 + stem_tol; // Całkowita długość krzyża
    
    difference() {
        translate([0, 0, 3]) cube([6, 5, 3.6], center = true);
        
        translate([0, 0, 3.5+2]) { // Pozycjonujemy stem na dole keycapa
            union() {
                // Poziome ramiona krzyża
                cube([stem_total_length, stem_arm_width, stem_height + 4], center = true);
                // Pionowe ramiona krzyża
                cube([stem_arm_width, stem_total_length, stem_height + 4], center = true);
            }
        }
    }
    
    // --- Główna bryła keycapa ---
    difference() {
        // 1. Podstawowa bryła keycapa
        // Używamy minkowski sumy dla zaokrąglonych krawędzi (opcjonalnie)
        // LUB po prostu cube z zaokrąglonymi narożnikami
        
        // Bryła pozytywna (zaokrąglony sześcian)
        // Używamy cylinder_fn = $fn/2 dla lepszego renderowania zaokrągleń
        hull() {
            translate([-(u_size/2 - corner_radius), -(u_size/2 - corner_radius), 0])
            cylinder(r = corner_radius, h = height, $fn = $fn/2);

            translate([(u_size/2 - corner_radius), -(u_size/2 - corner_radius), 0])
            cylinder(r = corner_radius, h = height, $fn = $fn/2);

            translate([-(u_size/2 - corner_radius), (u_size/2 - corner_radius), 0])
            cylinder(r = corner_radius, h = height, $fn = $fn/2);

            translate([(u_size/2 - corner_radius), (u_size/2 - corner_radius), 0])
            cylinder(r = corner_radius, h = height, $fn = $fn/2);
        }

        // Dodatkowe wycięcie dla skośnych boków (opcjonalnie)
        // Tworzy lekki "bevel" na górnej krawędzi
        if (top_offset > 0) {
            translate([0, 0, height/2 + 2]) // Lekko podnieść, żeby nie tworzyć artefaktów
            cube([u_size - top_offset * 2, u_size - top_offset * 2, height], center = true);
        }

        // 2. Wycięcie na MX Stem (krzyż)
        // To jest najważniejsza część - odwrotność trzpienia przełącznika
        translate([0, 0, 3.5+2]) { // Pozycjonujemy stem na dole keycapa
            union() {
                // Poziome ramiona krzyża
                cube([stem_total_length, stem_arm_width, stem_height + 4], center = true);
                // Pionowe ramiona krzyża
                cube([stem_arm_width, stem_total_length, stem_height + 4], center = true);
            }
        }
    }
}

module mx_keycap2(u_size = 23.1, y_size = 23.1, height = 8, top_offset = 1.5, corner_radius = 1.5, stem_height = 4.5, stem_tol = 0.1, character="") {
    
    // --- Definicje wymiarów MX Stem (Standardowe) ---
    // Krzyżowy trzpień MX ma boki ok. 3.9mm, a 'puste' przestrzenie ok. 1.2mm
    // Wymaga to precyzyjnego modelowania przestrzeni negatywnej.
    
    stem_arm_width = 1.2 + stem_tol;  // Szerokość "ramion" krzyża (przestrzeń negatywna)
    stem_total_length = 4.0 + stem_tol; // Całkowita długość krzyża
    //translate([0, -1, 0]) cylinder(h=50, r=1, center=true);
    translate([0, -1, 0]) difference() {
        translate([0, 0, 3]) cube([6, 5, 3.6], center = true);
        
        translate([0, 0, 3.5+2]) { // Pozycjonujemy stem na dole keycapa
            union() {
                // Poziome ramiona krzyża
                cube([stem_total_length, stem_arm_width, stem_height + 4], center = true);
                // Pionowe ramiona krzyża
                cube([stem_arm_width, stem_total_length, stem_height + 4], center = true);
            }
        }
    }
    
    // --- Główna bryła keycapa ---
    difference() {
        // 1. Podstawowa bryła keycapa
        // Używamy minkowski sumy dla zaokrąglonych krawędzi (opcjonalnie)
        // LUB po prostu cube z zaokrąglonymi narożnikami
        
        // Bryła pozytywna (zaokrąglony sześcian)
        // Używamy cylinder_fn = $fn/2 dla lepszego renderowania zaokrągleń
        union() {
            hull() {
                translate([-(u_size/2 - corner_radius), -(y_size/2 - corner_radius), 0])
                cylinder(r = corner_radius, h = height, $fn = $fn/2);

                translate([(u_size/2 - corner_radius), -(y_size/2 - corner_radius), 0])
                cylinder(r = corner_radius, h = height, $fn = $fn/2);

                translate([-(u_size/2 - corner_radius), (y_size/2 - corner_radius) + 0.5, 0])
                cylinder(r = corner_radius, h = height, $fn = $fn/2);

                translate([(u_size/2 - corner_radius), (y_size/2 - corner_radius) + 0.5, 0])
                cylinder(r = corner_radius, h = height, $fn = $fn/2);
            };
            
            
            translate([u_size / -2, y_size / 2, corner_radius]) 
                rotate([0, 90, 0]) {
                    difference() {
                        hull() {
                            translate([-10.5,-1,0]) cube([12, corner_radius+1, u_size]);
                            
//                            cylinder(r = corner_radius, h = u_size, $fn = $fn/2);
                        
                            translate([-12,1,0]) cylinder(r = corner_radius, h = u_size, $fn = $fn/2);
                        }
                        translate([-11,2,0]) cube([1, 1, u_size+1]);
                        translate([-9.5,2,0]) cube([1, 1, u_size+1]);
                    }
                }
                
        }
            //translate([0, 0, 0.2]) rotate([180, 0, 0]) linear_extrude(height = 0.2) text( character, size=6, valign="center", halign="center", font="Symbola:style=Bold");
        
        // Dodatkowe wycięcie dla skośnych boków (opcjonalnie)
        // Tworzy lekki "bevel" na górnej krawędzi
        if (top_offset > 0) {
            translate([0, 0, height/2 + 2]) // Lekko podnieść, żeby nie tworzyć artefaktów
            cube([u_size - top_offset * 2, y_size - top_offset * 2, height], center = true);
        }

        // 2. Wycięcie na MX Stem (krzyż)
        // To jest najważniejsza część - odwrotność trzpienia przełącznika
        translate([0, -1, 3.5+2]) { // Pozycjonujemy stem na dole keycapa
            union() {
                // Poziome ramiona krzyża
                cube([stem_total_length, stem_arm_width, stem_height + 4], center = true);
                // Pionowe ramiona krzyża
                cube([stem_arm_width, stem_total_length, stem_height + 4], center = true);
            }
        }
    }
            //color("red") translate([0, 0, 0.2]) rotate([180, 0, 0]) linear_extrude(height = 0.2) text( character, size=6, valign="center", halign="center", font="Symbola:style=Bold");
}

base_width = 104.4;

translate([0, 0, 14]) rotate([180,0,0]) {
    translate([-35,0,0]) mx_keycap2(u_size = 28, character = "PLAY");
    translate([0,0,0]) mx_keycap2(u_size = 40, character = "NEXT");
    translate([35,0,0]) mx_keycap2(u_size = 28, character = "LIKE");
};
// Keycap z większym zaokrągleniem i mniejszą tolerancją stemu
// translate([25,0,0])
// mx_keycap(corner_radius = 2.5, stem_tol = 0.05);

// Keycap 2U (np. Shift) - wymaga dostosowania u_size
// translate([50,0,0])
// mx_keycap(u_size = 37.05, stem_tol = 0.08); // 37.05 = 2 * 19.05 - 1.05 (między klawiszami)