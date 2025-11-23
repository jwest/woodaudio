/*
 * Parametryczny moduł gniazda (socketu) dla przełącznika typu MX
 * Wersja z pełnym dnem i otworami na piny (standard 5-pin / PCB Mount).
 *
 * Orientacja:
 * Oś X: Wzdłuż klipsów mocujących (wymiar 15.8mm).
 * Oś Y: Wzdłuż boków 14mm.
 * Oś Z: Wysokość.
 */

// Ustawienie domyślnej rozdzielczości
$fn = 50;

/**
 * Generuje pojedynczy socket MX z pełnym dnem i otworami na piny.
 *
 * @param u Szerokość i głębokość jednostki (standard 1U = 19.05mm)
 * @param h Wysokość całkowita socketu
 * @param plate_th Grubość "płytki" którą łapią klipsy (krytyczny wymiar, standard to 1.5mm)
 * @param tol Tolerancja druku 3D (dla korpusu przełącznika)
 * @param pin_tol Tolerancja druku 3D (dla otworów na piny)
 */
module mx_socket_solid_bottom(u = 19.05, h = 5.5, plate_th = 1.5, tol = 0, pin_tol = 0) {
    
    // --- Definicje wymiarów wycięcia (Korpus) ---
//    u=20;
    // 1. Górny otwór (standard 14x14mm)
    plate_cut_x = 14 + tol;
    plate_cut_y = 14 + tol;
    
    // 2. Wycięcie na klipsy (poniżej płytki)
    // Klipsy są na osi X, więc tam wymiar jest większy (~15.8mm)
    clip_cut_x = 15.8 + tol; 
    clip_cut_y = 14 + tol;   // Oś Y pozostaje 14mm
    
    // 3. Głębokość korpusu switcha poniżej płytki
    body_depth = 3.5;
    
    // --- Definicje wymiarów (Piny) ---
    // Średnice otworów na piny
    d_central = 4.0 + pin_tol;   // Centralny bolec
    d_pin = 1.6 + pin_tol;       // Piny kontaktowe (metalowe)
    d_plastic = 1.7 + pin_tol;   // Piny stabilizujące (plastikowe)

    // Pozycje pinów (standardowy footprint KiCad, względem [0,0,0])
    pos_pin1 = [-2.54, -5.08, 0];   // Pin metalowy 1
    pos_pin2 = [3.81, -2.54, 0];    // Pin metalowy 2 (asymetryczny)
    pos_plastic1 = [-5.08, 0, 0];   // Pin stabilizujący 1
    pos_plastic2 = [5.08, 0, 0];    // Pin stabilizujący 2

    // --- Budowa Modułu ---
    // Używamy zagnieżdżonej operacji difference()
    
    difference() {
        
        // KROK 1: Stwórz bryłę socketu z wycięciem na korpus, ale z pełnym dnem
        difference() {
            
            // 1a. Bryła pozytywna (Solid)
            // Główny sześcian 1U
            cube([u, u, h], center = true);
            
            // 1b. Przestrzeń negatywna (Korpus)
            // Wycięcie na górną część przełącznika
            union() {
                // Górny otwór (14x14) na głębokość `plate_th`
                translate([0, 0, (h / 2) - (plate_th / 2)])
                cube([plate_cut_x, plate_cut_y, plate_th + 0.01], center = true);

                // Komora na klipsy (15.8 x 14) na głębokość `body_depth`
                translate([0, 0, (h / 2) - plate_th - (body_depth / 2)])
                cube([clip_cut_x, clip_cut_y, body_depth], center = true);
            }
        }
        
        // KROK 2: Odejmij otwory na piny od bryły stworzonej w Kroku 1
        // Używamy cylindrów wyższych niż 'h' aby zapewnić pełne przebicie
        union() {
            // Otwór centralny (pozycja [0,0,0])
            cylinder(h = h + 2, d = d_central, center = true);
            
            // Otwory na piny (kontakty)
            translate(pos_pin1)
            cylinder(h = h + 2, d = d_pin, center = true);
            
            translate(pos_pin2)
            cylinder(h = h + 2, d = d_pin, center = true);
            
            // Otwory na piny (plastikowe, stabilizujące)
            translate(pos_plastic1)
            cylinder(h = h + 2, d = d_plastic, center = true);
            
            translate(pos_plastic2)
            cylinder(h = h + 2, d = d_plastic, center = true);
        }
    }
}

// --- Przykładowe użycie ---

// Pojedynczy socket z pełnym dnem

base_width = 104.4;

translate([0, 1, 0]) { difference() {
    translate([0, 0, 0]) cube([base_width - 14, 18, 5.5], center = true);
    union() { 
        translate([-35, 0, 0]) cube([18, 18, 18], center = true);
        translate([0, 0, 0]) cube([18, 18, 18], center = true);
        translate([35, 0, 0]) cube([18, 18, 18], center = true);
    }
    
}

translate([-35, 0, 0]) mx_socket_solid_bottom(u = 18);
translate([0, 0, 0]) mx_socket_solid_bottom(u = 18);
translate([35, 0, 0]) mx_socket_solid_bottom(u = 18);
}