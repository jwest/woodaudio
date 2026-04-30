include <BOSL2/std.scad>
$fn=128;

hole_screw_r = 1.25;
hole_screw_head_r = 3.2;

display_width = 104.65;
display_height = 65.5;
display_weight = 1.1;
display_radious = 4.0;

pi_hole_width=58.0;
pi_hole_height=49.0;

display_plate_margin = 3;
display_plate_margin_left_inner = 1.5;
display_plate_width = display_width - 2 * display_plate_margin;
display_plate_height = display_height - 2 * display_plate_margin;
display_plate_weight = 6.7;

display_pin_r = 5.5 / 2;
display_pin_h = 7.8;

display_pin_margin_top = 5.5;
display_pin_margin_bottom = 4.8;
display_pin_margin_left = 15.2;
display_pin_margin_right = 25.4;

display_pin_1_margin_left = display_pin_margin_right + display_pin_r + 0.25;
display_pin_1_margin_bottom = display_pin_margin_bottom + display_pin_r + 0.25;
display_pin_2_margin_left = display_pin_1_margin_left;
display_pin_2_margin_bottom = display_height - display_pin_margin_top - display_pin_r - 0.25;

display_pin_3_margin_left = display_pin_margin_right + display_pin_r + 58; //57.6
display_pin_3_margin_bottom = display_pin_1_margin_bottom;
display_pin_4_margin_left = display_pin_3_margin_left;
display_pin_4_margin_bottom = display_pin_2_margin_bottom;

display_mount_pin_base_size = (display_pin_r * 2 + 2.5) / 2;
display_mount_pin_base_height = 9;
display_mount_pin_base_hole_size = 6 / 2;
display_mount_pin_base_deep = display_plate_weight + 1.5;

display_border = 2.0;

keys_count = 3;
keys_space = 1;
keys_weight = 2;
keys_rounding = 3;
keys_width = 20;
keys_height = display_height - 2 * display_border - 2 * keys_space;
keys_margin_bottom = keys_space + display_border;
keys_margin_left = display_width + keys_space;
keys_steam_length = 3.6;

keys_mount_width = keys_width + 2 * keys_space;
keys_mount_height = display_height - 2 * display_border;
keys_mount_deep = keys_weight + keys_steam_length + 6.65;
keys_mount_margin_bottom = display_border;
keys_mount_margin_left = display_width;
keys_mount_weight = 5.5 + 1.5; // 5.5 is min value

front_panel_width = display_width + display_border + keys_width + 2 * keys_space;
front_panel_height = display_height;

pi_plate_margin_bottom = 18;
pi_plate_margin_left = display_pin_margin_right + 0.4;
pi_plate_margin_front = 19.25+6;
pi_plate_weight = 3;
pi_plate_hole_margin = 3.2;
pi_plate_deep = pi_hole_height + 6.4;
pi_plate_height = pi_hole_width + 6.4;
pi_plate_wall = pi_plate_hole_margin * 2;
pi_plate_screw_h = 4;
pi_plate_power_pi_margin_left = 41;

case_h = pi_plate_margin_front + pi_plate_deep + 6.5; //30;//19.25;//80;
case_front_h = display_mount_pin_base_deep + display_mount_pin_base_height;
case_screw_deep = case_h - 21.25 - 2;
case_screw_margin_front = case_h - case_screw_deep / 2 - 4;
case_screw_margin_left = display_border + hole_screw_head_r - 0.3;
case_screw_margin_bottom = case_screw_margin_left;
echo (case_front_h=case_front_h);

case_top_margin_front = 19.25;

/**
 * SIMULATIONS
 */
module simulate_display_pin() {
    ycyl(
        r = display_pin_r,
        h = display_pin_h,
        anchor = FRONT
    );
}
module simulate_display(edges = [TOP+LEFT, TOP+RIGHT, BOT+LEFT, BOT+RIGHT], display_size_tol = 0) {
    color("red") translate([display_size_tol / -2, display_size_tol / -2, display_size_tol / -2]) cuboid(
        [display_width + display_size_tol, display_weight + display_size_tol, display_height + display_size_tol],
        rounding=display_radious,
        anchor=FRONT+LEFT+BOT,
        edges=edges
    ){
            attach(BACK, FRONT) cuboid(
                [display_plate_width, display_plate_weight, display_plate_height]
            ) {
                    position(BACK+TOP+RIGHT) down(display_pin_margin_top) left(display_pin_margin_left) simulate_display_pin();
                    position(BACK+TOP+LEFT) down(display_pin_margin_top) right(display_pin_margin_right) simulate_display_pin();
                    position(BACK+BOTTOM+RIGHT) up(display_pin_margin_bottom) left(display_pin_margin_left) simulate_display_pin();
                    position(BACK+BOTTOM+LEFT) up(display_pin_margin_bottom) right(display_pin_margin_right) simulate_display_pin();
            }
    }
}

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

module keycup_mount() {
    // MX Stem - Krzyżowy trzpień MX ma boki ok. 3.9mm, a 'puste' przestrzenie ok. 1.2mm (2.6)
    stem_height = 4.5;
    stem_tol = 0.1;
    stem_arm_width = 1.2 + stem_tol;  // Szerokość "ramion" krzyża (przestrzeń negatywna)
    stem_total_length = 4.0 + stem_tol; // Całkowita długość krzyża
    
    difference() {
        translate([0, 0, 3]) cube([6, 5, keys_steam_length], center = true);
        
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

module mx_socket() {
    u = 18;
    h = 5.5; 
    plate_th = 1.5; 
    tol = 0;
    pin_tol = 0;

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

module buttons() {
    translate([0, 0, 0]) translate([keys_margin_left, 0, keys_margin_bottom]) {
        difference() {
            union() {
                cuboid(
                    [keys_width, keys_weight, keys_height],
                    rounding=keys_rounding,
                    anchor=FRONT+LEFT+BOT,
                    edges=[TOP+RIGHT, BOT+RIGHT]
                );
                for (i=[1 : 2 : keys_count * 2]) {
                    translate([keys_width / 2, keys_weight / 2 - 0.2, (i * keys_height) / (keys_count * 2)])
                        rotate([-90, 90, 0])
                            keycup_mount();
                }
            }

            for (i=[1:keys_count]) {
                translate([0, 0, i * keys_height / keys_count ]) cuboid(
                    [keys_width, keys_weight, keys_space],
                    anchor=FRONT+LEFT
                );
            }
        };
    }    
}

module buttons_mount() {
    color("blue") translate([keys_mount_margin_left, keys_mount_deep, keys_mount_margin_bottom]) {
        difference() {
            union() {
                difference() {
                    union() {
                        cuboid(
                            [keys_mount_width, keys_mount_weight, keys_mount_height],
                            rounding=keys_rounding,
                            anchor=FRONT+LEFT+BOT,
                            edges=[TOP+RIGHT, BOT+RIGHT]
                        );
                        translate([-display_plate_margin_left_inner, -keys_mount_deep + display_weight, 0]) cuboid(
                            [display_plate_margin_left_inner, keys_mount_deep + keys_mount_weight - display_weight, keys_mount_height],
                            anchor=FRONT+LEFT+BOT
                        );
                    }
                    translate([keys_space, 0, keys_space]) for (i=[1 : 2 : keys_count * 2]) {
                        translate([keys_width / 2, keys_weight / 2 + 1.75, (i * (keys_height)) / (keys_count * 2)])
                            rotate([90, 90, 0]) 
                                cube([18, 18, keys_mount_weight-1.5], center=true);
                    }
                }
                
                translate([keys_space, 0, keys_space]) for (i=[1 : 2 : keys_count * 2]) {
                    translate([keys_width / 2, keys_weight / 2 + 1.75, (i * keys_height) / (keys_count * 2)])
                        rotate([90, 90, 0]) 
                            mx_socket();
                }
            }
            
            translate([keys_space, 3, keys_space]) for (i=[1 : 2 : keys_count * 2]) {
                translate([keys_width / 2, keys_weight / 2 + 1.75, (i * keys_height) / (keys_count * 2)])
                    cube([14, 3, 15.8], center = true);
            }
        };
    }    
}

module case() {
    difference() {
        union() {
            rect_tube(
                size=[front_panel_width, display_height], 
                wall=display_border, 
                rounding=display_radious,
                irounding=display_radious - display_radious * 0.25,
                h=case_h,
                anchor=BACK+LEFT+BOT,
                orient=BACK
            );
            translate([case_screw_margin_left, case_screw_margin_front, case_screw_margin_bottom]) {
                difference() {
                    cuboid([hole_screw_head_r * 2, case_screw_deep, hole_screw_head_r * 2], rounding = hole_screw_head_r, edges=RIGHT+TOP);
                    ycyl(r = hole_screw_r, h=case_screw_deep + 0.1);
                }
            }
            
            translate([case_screw_margin_left, case_screw_margin_front, display_height - case_screw_margin_bottom]) {
                difference() {
                    cuboid([hole_screw_head_r * 2, case_screw_deep, hole_screw_head_r * 2], rounding = hole_screw_head_r, edges=RIGHT+BOTTOM);
                    ycyl(r = hole_screw_r, h=case_screw_deep + 0.1);
                }
            }
            
            translate([front_panel_width - case_screw_margin_left, case_screw_margin_front, case_screw_margin_bottom]) {
                difference() {
                    cuboid([hole_screw_head_r * 2, case_screw_deep, hole_screw_head_r * 2], rounding = hole_screw_head_r, edges=LEFT+TOP);
                    ycyl(r = hole_screw_r, h=case_screw_deep + 0.1);
                }
            }
            
            translate([front_panel_width - case_screw_margin_left, case_screw_margin_front, display_height - case_screw_margin_bottom]) {
                difference() {
                    cuboid([hole_screw_head_r * 2, case_screw_deep, hole_screw_head_r * 2], rounding = hole_screw_head_r, edges=LEFT+BOTTOM);
                    ycyl(r = hole_screw_r, h=case_screw_deep + 0.1);
                }
            }
            
            //case top mount            
            translate([0, case_top_margin_front - 3, display_height - 2 * display_border])
                cube([front_panel_width, 6, display_border]);
            
            translate([6, case_top_margin_front - 4, display_height - 2 * display_border])
                cube([6, case_screw_deep, display_border]);
            
            translate([front_panel_width - 6 - 6, case_top_margin_front - 4, display_height - 2 * display_border])
                cube([6, case_screw_deep, display_border]);
        }
        simulate_display(edges = [TOP+RIGHT, BOT+RIGHT], display_size_tol=0.1);
        for (i=[2:1.6:display_height]) {
            translate([0, 0, i])
                cube([1,case_h,0.8]);
            translate([front_panel_width - 1, 0, i])
                cube([1,case_h,0.8]);
            
            if (i > (display_height / 8) * 2 && i < (display_height / 8) * 6) {
                translate([1, 0, i]) {
                    translate([0, (case_h / 16) * 4, 0])
                        cube([1,case_h / 8,0.8]);
                    
                    translate([0, (case_h / 16) * 8, 0])
                        cube([1,case_h / 8,0.8]);
                    
                    translate([0, (case_h / 16) * 12, 0])
                        cube([1,case_h / 8,0.8]);
                }
                
                translate([front_panel_width - 2, 0, i]) {
                    translate([0, (case_h / 16) * 4, 0])
                        cube([1,case_h / 8,0.8]);
                    
                    translate([0, (case_h / 16) * 8, 0])
                        cube([1,case_h / 8,0.8]);
                    
                    translate([0, (case_h / 16) * 12, 0])
                        cube([1,case_h / 8,0.8]);
                }
            }
            //translate([front_panel_width - 0.8, 0, i])
            //    cube([0.8,case_h,1]);
            
            //if (i< (case_h-(21 * 2))) {
            //    translate([1, 32, i+1]) {
            //        cube([0.8, (case_h / 2), 1]);
            //    }
            //}
        }; 
        translate([0, case_top_margin_front, display_height - display_border])
            cube([front_panel_width, case_h - case_top_margin_front + 1, display_border]);
    }
}

module case_top() {
    front_margin_for_bottom_case = case_screw_margin_front + (case_screw_deep / 2) + 0.1;
    lock_size = 10;
    difference() {
        union() {
            translate([15, front_margin_for_bottom_case - 3, display_height - 2 * display_border]) cube([front_panel_width - 30, 6, 4]);
            
            translate([front_panel_width / 2 + 30, 19.25 + (lock_size / 2), display_height - 3 * display_border + (display_border / 2)]) cube([lock_size, lock_size, display_border], center=true);
            translate([front_panel_width / 2 + 30, 22.5 + (lock_size / 2), display_height - 2 * display_border + (display_border / 2)]) cube([lock_size, lock_size, display_border], center=true);
            
            translate([front_panel_width / 2 - 30, 19.25 + (lock_size / 2), display_height - 3 * display_border + (display_border / 2)]) cube([lock_size, lock_size, display_border], center=true);
            translate([front_panel_width / 2 - 30, 22.5 + (lock_size / 2), display_height - 2 * display_border + (display_border / 2)]) cube([lock_size, lock_size, display_border], center=true);
            
            difference() {
                union() {
                    rect_tube(
                        size=[front_panel_width, display_height], 
                        wall=display_border, 
                        rounding=display_radious,
                        irounding=display_radious - display_radious * 0.25,
                        h=case_h,
                        anchor=BACK+LEFT+BOT,
                        orient=BACK
                    );            
                }
                rect_tube(
                    size=[front_panel_width, display_height - display_border], 
                    wall=display_border, 
                    rounding=0,
                    irounding=display_radious - display_radious * 0.25,
                    h=case_h,
                    anchor=BACK+LEFT+BOT,
                    orient=BACK
                );
            }
            color("pink") translate([display_border, front_margin_for_bottom_case, display_border]) cuboid(
                [front_panel_width - 2 * display_border, display_border, display_height - 2 * display_border],
                rounding = display_radious - display_radious * 0.25,
                anchor = FRONT+LEFT+BOT,
                edges = [TOP+LEFT, TOP+RIGHT, BOT+LEFT, BOT+RIGHT]
            ); 
        };
        //case();
        translate([0, 0, display_height - display_border])
            cube([front_panel_width, 19.25, display_border]);
        
        translate([case_screw_margin_left, front_margin_for_bottom_case + (display_border / 2), case_screw_margin_bottom]) {
            ycyl(r = hole_screw_r, h=display_border + 0.1);
        }
        
        translate([case_screw_margin_left, front_margin_for_bottom_case + (display_border / 2), display_height - case_screw_margin_bottom]) {
            ycyl(r = hole_screw_r, h=display_border + 0.1);
        }
        
        translate([front_panel_width - case_screw_margin_left, front_margin_for_bottom_case + (display_border / 2), case_screw_margin_bottom]) {
            ycyl(r = hole_screw_r, h=display_border + 0.1);
        }
        
        translate([front_panel_width - case_screw_margin_left, front_margin_for_bottom_case + (display_border / 2), display_height - case_screw_margin_bottom]) {
            ycyl(r = hole_screw_r, h=display_border + 0.1);
        }
        
        translate([display_border, front_margin_for_bottom_case, display_border]) {
            // optic + cinch
            translate([31,0,39]) cuboid([23 , display_border + 0.1, 12], rounding = 1, anchor = FRONT+LEFT+BOT, edges = [TOP+LEFT, TOP+RIGHT, BOT+LEFT, BOT+RIGHT]);
            
            //usb
            translate([44,0,22]) cuboid([39 , display_border + 0.1, 7], rounding = 1, anchor = FRONT+LEFT+BOT, edges = [TOP+LEFT, TOP+RIGHT, BOT+LEFT, BOT+RIGHT]);
        }
    }
}

module display_plate() {
    pin_r = display_mount_pin_base_size;
    pin_hole_r = display_mount_pin_base_hole_size;
    pin_h = display_mount_pin_base_height;
    pin_deep = display_mount_pin_base_deep;
    screw_h = 15;
    screw_head_h = 1; // 1.5
    screw_head_y = 10.75; // 10
    
    difference() {
        union() {
            translate([display_border / 2,14.25, display_border / 2]) rotate([-90,0,0]) rect_tube(
                size = [display_width - display_border, display_height - display_border],
                wall = pin_r - display_border / 2,
                h = 5,
                anchor=BACK+LEFT+BOT,
                rounding=[0, display_radious, display_radious, 0]
            );
            
            translate([display_pin_1_margin_left - pin_r,16.25,0]) cube(
                [8, 3, display_height],
                anchor=BACK+LEFT+BOT,
                center = false
            );
            
            translate([display_pin_3_margin_left - pin_r,16.25,0]) cube(
                [8, 3, display_height],
                anchor=BACK+LEFT+BOT,
                center = false
            );
            
            translate([display_pin_1_margin_left, pin_deep, display_pin_1_margin_bottom])
                ycyl(r=pin_r, h=pin_h,center=false);
            translate([display_pin_2_margin_left, pin_deep, display_pin_2_margin_bottom])
                ycyl(r=pin_r, h=pin_h,center=false);
            translate([display_pin_3_margin_left, pin_deep, display_pin_3_margin_bottom])
                ycyl(r=pin_r, h=pin_h,center=false);
            translate([display_pin_4_margin_left, pin_deep, display_pin_4_margin_bottom])
                ycyl(r=pin_r, h=pin_h,center=false);
        };
    
        union() {
            // holes
            translate([display_pin_1_margin_left, pin_deep-0.1, display_pin_1_margin_bottom]) {
                ycyl(r=pin_hole_r, h=pin_h+0.2,center=false);
                ycyl(r=hole_screw_r, h=screw_h,center=false);                
                translate([0,screw_head_y,0]) ycyl(r=hole_screw_head_r, h=screw_head_h,center=false);
            }
            translate([display_pin_2_margin_left, pin_deep-0.1, display_pin_2_margin_bottom]) {
                ycyl(r=pin_hole_r, h=pin_h+0.2,center=false);
                ycyl(r=hole_screw_r, h=screw_h,center=false);                
                translate([0,screw_head_y,0]) ycyl(r=hole_screw_head_r, h=screw_head_h,center=false);
            }
            translate([display_pin_3_margin_left, pin_deep-0.1, display_pin_3_margin_bottom]) {
                ycyl(r=pin_hole_r, h=pin_h+0.2,center=false);
                ycyl(r=hole_screw_r, h=screw_h,center=false);               
                translate([0,screw_head_y,0]) ycyl(r=hole_screw_head_r, h=screw_head_h,center=false); 
            }
            translate([display_pin_4_margin_left, pin_deep-0.1, display_pin_4_margin_bottom]) {
                ycyl(r=pin_hole_r, h=pin_h+0.2,center=false);
                ycyl(r=hole_screw_r, h=screw_h,center=false);               
                translate([0,screw_head_y,0]) ycyl(r=hole_screw_head_r, h=screw_head_h,center=false); 
            }
            
        };
    };
}

module pi_mount() {
    color("red") difference() {
        union() {
            translate([pi_plate_margin_left, pi_plate_margin_front, pi_plate_margin_bottom]) {
                rect_tube(
                    size = [pi_plate_height, pi_plate_deep],
                    wall = pi_plate_wall,
                    h = pi_plate_weight,
                    anchor=FRONT+LEFT+BOT,
                    rounding=pi_plate_hole_margin,
                    irounding=pi_plate_hole_margin,
                    orient=UP
                );
                translate([pi_plate_hole_margin, pi_plate_hole_margin, pi_plate_screw_h]) {  
                    for (x = [0:1:1]) for (y = [0:1:1])
                        translate([x * pi_hole_width, y * pi_hole_height, 0]) 
                            cyl(r = hole_screw_head_r, h=pi_plate_weight+0.1);
                }
            }
        };
        translate([pi_plate_margin_left, pi_plate_margin_front, pi_plate_margin_bottom + (pi_plate_weight + pi_plate_screw_h) / 2]) {
            translate([pi_plate_hole_margin, pi_plate_hole_margin, 0]) {
                for (x = [0:1:1]) for (y = [0:1:1])
                    translate([x * pi_hole_width, y * pi_hole_height, 0]) cyl(r = hole_screw_r, h=pi_plate_weight+ pi_plate_screw_h +0.1);
            }
        };
        translate([pi_plate_margin_left + pi_plate_power_pi_margin_left, pi_plate_margin_front + pi_plate_deep - 3, 0])
            cube([10,pi_plate_wall, pi_plate_margin_bottom + pi_plate_weight], center=false);
    };
}

module pi_plate() {
    difference() {
        union() {
            translate([pi_plate_margin_left, pi_plate_margin_front, pi_plate_margin_bottom]) {
                translate([pi_plate_wall, -2, -1 * pi_plate_margin_bottom]) rect_tube(
                    size = [pi_plate_height - 2 * pi_plate_wall, pi_plate_deep + 4],
                    wall = pi_plate_wall,
                    h = pi_plate_margin_bottom + pi_plate_weight,
                    anchor=FRONT+LEFT+BOT,
                    rounding=pi_plate_hole_margin,
                    irounding=pi_plate_hole_margin,
                    orient=UP
                );
            }
        };
        translate([pi_plate_margin_left + pi_plate_power_pi_margin_left, pi_plate_margin_front + pi_plate_deep - 7, 0])
            cube([10,pi_plate_wall + 3, pi_plate_margin_bottom + pi_plate_weight + 1], center=false);
        translate([pi_plate_margin_left + 2 * pi_plate_wall, pi_plate_margin_front - pi_plate_wall, 0])
            cube([38.8, 3 * pi_plate_wall, pi_plate_margin_bottom - pi_plate_weight], center=false);
        pi_mount();
    };
}

difference() {
    union() {
        //case();
        case_top();
        //buttons();
        //buttons_mount();
        //display_plate();
        //pi_plate();
        //pi_mount();

        //simulate_display();
    };
    
    //clear buttons
    //translate([display_width, 0, 0]) cube([40, 40, display_height], center=false);
}

//ruler();