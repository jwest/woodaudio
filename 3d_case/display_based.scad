include <BOSL2/std.scad>
$fn=128;

display_width = 104.35;
display_height = 65.5;
display_weight = 1.1;
display_radious = 4.0;

display_plate_margin = 3;
display_plate_width = display_width - 2 * display_plate_margin;
display_plate_height = display_height - 2 * display_plate_margin;
display_plate_weight = 6.7;

display_pin_r = 5.5 / 2;
display_pin_h = 7.8;

display_pin_margin_top = 5.7;
display_pin_margin_bottom = 4.8;
display_pin_margin_left = 25.3;
display_pin_margin_right = 15.7;

// width 104.35
// 25.3
// 5.5 pin
// 52.4
// 5.5 pin
// 15.7

// height 65.5
// 5.7
// 5.5 pin
// 43.4
// 5.5 pin
// 4.8

// weight
// 1.1 display
// 6.7 plate
// 7.8 pin

// plate border 1.5 (top/bottom 3, left 1.5, right 3.8)

module display_pin() {
    ycyl(
        r = display_pin_r,
        h = display_pin_h,
        anchor = FRONT
    );
}

module display() {
    tag("remove") color("grey") attachable() {
        // display
        cuboid(
            [display_width, display_weight, display_height], 
            anchor=FRONT+LEFT+BOT,
            rounding=display_radious, 
            edges=[TOP+LEFT, TOP+RIGHT, BOT+LEFT, BOT+RIGHT]
        ) {
                attach(BACK, FRONT) cuboid(
                    [display_plate_width, display_plate_weight, display_plate_height]
                ) {
                        position(BACK+TOP+RIGHT) down(display_pin_margin_top) left(display_pin_margin_left) display_pin();
                        position(BACK+TOP+LEFT) down(display_pin_margin_top) right(display_pin_margin_right) display_pin();
                        position(BACK+BOTTOM+RIGHT) up(display_pin_margin_bottom) left(display_pin_margin_left) display_pin();
                        position(BACK+BOTTOM+LEFT) up(display_pin_margin_bottom) right(display_pin_margin_right) display_pin();
                }
        }
        
        children();
    }
}

// MX Socket
module mx_socket_solid() {
    
    // --- Definicje wymiarów wycięcia (Korpus) ---
    u = 19.05;      // Szerokość i głębokość jednostki (standard 1U = 19.05mm)
    h = 5.5;        // Wysokość całkowita socketu
    plate_th = 1.5; // plate_th Grubość "płytki" którą łapią klipsy (krytyczny wymiar, standard to 1.5mm)
    
    // 1. Górny otwór (standard 14x14mm)
    plate_cut_x = 14;
    plate_cut_y = 14;
    
    // 2. Wycięcie na klipsy (poniżej płytki)
    // Klipsy są na osi X, więc tam wymiar jest większy (~15.8mm)
    clip_cut_x = 15.8; 
    clip_cut_y = 14;   // Oś Y pozostaje 14mm
    
    // 3. Głębokość korpusu switcha poniżej płytki
    body_depth = 3.5;
    
    // --- Definicje wymiarów (Piny) ---
    // Średnice otworów na piny
    d_central = 4.0;   // Centralny bolec
    d_pin = 1.6;       // Piny kontaktowe (metalowe)
    d_plastic = 1.7;   // Piny stabilizujące (plastikowe)

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

module keycup_mount() {
    // --- Definicje wymiarów MX Stem (Standardowe) ---
    // Krzyżowy trzpień MX ma boki ok. 3.9mm, a 'puste' przestrzenie ok. 1.2mm
    // Wymaga to precyzyjnego modelowania przestrzeni negatywnej.
    stem_height = 4.5;
    stem_tol = 0.1;
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
}

module case() {
    difference() {
        cuboid(
            [display_width + 20, display_weight, display_height], 
            anchor=FRONT+LEFT+BOT,
            rounding=display_radious, 
            edges=[TOP+LEFT, TOP+RIGHT, BOT+LEFT, BOT+RIGHT]
        )
            position(BACK+BOT+LEFT) rect_tube(
                size=[display_width + 20, display_height], 
                wall=1.5, 
                rounding=display_radious,
                irounding=display_radious - display_radious * 0.25,
                h=80,
                anchor=BACK+LEFT+BOT,
                orient=BACK
            );
        translate([0,-0.1,0]) display();
        //tag("remove") display();
    }
}

display();

case();