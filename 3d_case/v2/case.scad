include <BOSL2/std.scad>
$fn=64;
tol = 0.001;
tol2 = tol * 2;

use <display.scad>;

// wood case size = 7 x 8

display_width = 104.35;
display_height = 65.5;
display_weight = 1.1;
display_radious = 4.0;

display_plate_margin = 2;
display_plate_margin_left = 1.5;
            
mount_margin_left = 8;
mount_margin_right = 4;

case_margin_extra = 5;
case_margin_left = 6 + case_margin_extra;   // 6mm
case_margin_right = 2 + case_margin_extra;  // 10mm

case_width = display_width + case_margin_left + case_margin_right;

display_pin_r = 5.5 / 2;
display_pin_h = 7.8;

display_border = 2.0;

display_pin_margin_top = 5.5;
display_pin_margin_bottom = 4.8;
display_pin_margin_left = 15.2;
display_pin_margin_right = 25.4;

hole_screw_r = 1.25;
hole_screw_head_r = 3.2;

display_plate_margin_left_inner = 1.5;
display_plate_width = display_width - 2 * display_plate_margin;
display_plate_height = display_height - 2 * display_plate_margin;
display_plate_weight = 6.7;

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

pin_r = display_mount_pin_base_size;
pin_hole_r = display_mount_pin_base_hole_size;
pin_h = display_mount_pin_base_height;
pin_deep = display_mount_pin_base_deep;
screw_h = 15;
screw_head_h = 1; // 1.5
screw_head_y = 10.75; // 10

screw_insert_d = 3.7; //hole_diameter
screw_insert_m3_d = 4.5; //hole_diameter

module display_plate() {

    difference() {
        union() {
            // translate([display_border / 2,14.25, display_border / 2]) rotate([-90,0,0]) rect_tube(
            //     size = [display_width - display_border, display_height - display_border],
            //     wall = pin_r - display_border / 2,
            //     h = 5,
            //     anchor=BACK+LEFT+BOT,
            //     rounding=[0, display_radious, display_radious, 0]
            // );
            
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
                //translate([0,screw_head_y,0]) ycyl(r=hole_screw_head_r, h=screw_head_h,center=false);
            }

            translate([display_pin_2_margin_left, pin_deep-0.1, display_pin_2_margin_bottom]) {
                ycyl(r=pin_hole_r, h=pin_h+0.2,center=false);
                ycyl(r=hole_screw_r, h=screw_h,center=false);                
                //translate([0,screw_head_y,0]) ycyl(r=hole_screw_head_r, h=screw_head_h,center=false);
            }
            translate([display_pin_3_margin_left, pin_deep-0.1, display_pin_3_margin_bottom]) {
                ycyl(r=pin_hole_r, h=pin_h+0.2,center=false);
                ycyl(r=hole_screw_r, h=screw_h,center=false);               
                //translate([0,screw_head_y,0]) ycyl(r=hole_screw_head_r, h=screw_head_h,center=false); 
            }
            translate([display_pin_4_margin_left, pin_deep-0.1, display_pin_4_margin_bottom]) {
                ycyl(r=pin_hole_r, h=pin_h+0.2,center=false);
                ycyl(r=hole_screw_r, h=screw_h,center=false);               
                //translate([0,screw_head_y,0]) ycyl(r=hole_screw_head_r, h=screw_head_h,center=false); 
            }
            
            translate([display_pin_1_margin_left, pin_deep-0.1, display_pin_1_margin_bottom + 22]) {
                ycyl(r=pin_hole_r, h=pin_h+0.2,center=false);
                ycyl(r=hole_screw_r, h=screw_h,center=false);
            }
            translate([display_pin_2_margin_left, pin_deep-0.1, display_pin_2_margin_bottom - 10]) {
                ycyl(r=pin_hole_r, h=pin_h+0.2,center=false);
                ycyl(r=hole_screw_r, h=screw_h,center=false);                
            }
            translate([display_pin_3_margin_left, pin_deep-0.1, display_pin_3_margin_bottom + 22]) {
                ycyl(r=pin_hole_r, h=pin_h+0.2,center=false);
                ycyl(r=hole_screw_r, h=screw_h,center=false);               
            }
            translate([display_pin_4_margin_left, pin_deep-0.1, display_pin_4_margin_bottom - 10]) {
                ycyl(r=pin_hole_r, h=pin_h+0.2,center=false);
                ycyl(r=hole_screw_r, h=screw_h,center=false);               
            }
        };
    };
}


module front_case() {
    
    difference() {
        union() {
            back(0.1) cuboid(
                [display_width + case_margin_left + case_margin_right, display_height, display_weight],
                anchor=BACK+LEFT+BOT,
                orient=BACK
            );

            back(display_weight) rect_tube(
                size=[display_width + case_margin_left + case_margin_right, display_height], 
                wall=display_plate_margin, 
                rounding=0, //!!! for wood montage
                irounding=display_radious - display_radious * 0.25,
                h=18.15,
                anchor=BACK+LEFT+BOT,
                orient=BACK
            );
                
            right(case_margin_left) 
                display_plate();
            
            translate([5 + mount_margin_left / 2, 19.25, 16]) rotate([90, 0, 0]) {
                difference() {
                    cuboid([8 + mount_margin_left, 8, 4], anchor=[0, 0, -1], rounding=4, edges=[FWD+RIGHT, BACK+RIGHT]);
                    translate([mount_margin_left / 2, 0, 0]) cylinder(h = 5, d = screw_insert_d);
                }
            }
            translate([5 + mount_margin_left / 2, 19.25, 58]) rotate([90, 0, 0]) {
                difference() {
                    cuboid([8 + mount_margin_left, 8, 4], anchor=[0, 0, -1], rounding=4, edges=[FWD+RIGHT, BACK+RIGHT]);
                    translate([mount_margin_left / 2, 0, 0]) cylinder(h = 5, d = screw_insert_d);
                }
            }
            translate([case_width - 5 - mount_margin_right / 2, 19.25, 16]) rotate([90, 0, 0]) {
                difference() {
                    cuboid([8 + mount_margin_right, 8, 4], anchor=[0, 0, -1], rounding=4, edges=[FWD+LEFT, BACK+LEFT]);
                    translate([mount_margin_right / -2, 0, 0]) cylinder(h = 5, d = screw_insert_d);
                }
            }
            translate([case_width - 5 - mount_margin_right / 2, 19.25, 58]) rotate([90, 0, 0]) {
                difference() {
                    cuboid([8 + mount_margin_right, 8, 4], anchor=[0, 0, -1], rounding=4, edges=[FWD+LEFT, BACK+LEFT]);
                    translate([mount_margin_right / -2, 0, 0]) cylinder(h = 5, d = screw_insert_d);
                }
            }
            
            translate([0, 0.1, 0]) {
                cuboid([7,5,7], anchor=[-1,-1,-1], rounding=0, edges=[FWD+DOWN,BACK+DOWN]);
            }
            
            translate([0, 0.1, display_height - 7]) {
                cuboid([7,5,7], anchor=[-1,-1,-1], rounding=0, edges=[FWD+DOWN,BACK+DOWN]);
            }
            
            translate([case_width - 7, 0.1, 0]) {
                cuboid([7,5,7], anchor=[-1,-1,-1], rounding=0, edges=[FWD+DOWN,BACK+DOWN]);
            }
            
            translate([case_width - 7, 0.1, display_height - 7]) {
                cuboid([7,5,7], anchor=[-1,-1,-1], rounding=0, edges=[FWD+DOWN,BACK+DOWN]);
            }
        }
        
        for (x = [0, 1]) for(z = [0, 1]) {
            translate([3.6 + (x * (case_width - (3.6 * 2))), 9, 3.6 + (z * (display_height - (3.6 * 2)))]) {
                rotate([90,0,0]) cylinder(h=9, d=screw_insert_m3_d);
            }
        }

        right(case_margin_left) 
            display();
    }
    

    // back(display_weight + 0.1) rect_tube(
    //     size=[display_width, display_height], 
    //     wall=1.5, 
    //     rounding=display_radious,
    //     irounding=display_radious - display_radious * 0.25,
    //     h=display_weight,
    //     anchor=BACK+LEFT+BOT,
    //     orient=BACK
    // );

    // display_plate();

    // difference() {
    //     right(display_width - display_plate_margin) rect_tube(
    //         size=[buttons_panel_width + display_plate_margin, display_height], 
    //         wall=display_plate_margin, 
    //         rounding=[display_radious, 0, 0, display_radious],
    //         irounding=[display_radious - display_radious * 0.25, 0, 0, display_radious - display_radious * 0.25],
    //         h=display_weight,
    //         anchor=BACK+LEFT+BOT,
    //         orient=BACK
    //     );        
    //     // display
    //     color("red") cuboid(
    //         [display_width, display_weight, display_height], 
    //         anchor=FRONT+LEFT+BOT,
    //         rounding=display_radious, 
    //         edges=[TOP+LEFT, TOP+RIGHT, BOT+LEFT, BOT+RIGHT]
    //     );
    // }
}

// --- Parametry fizyczne RPi 3A+ ---
rpi_width = 65;       // Szerokość PCB
rpi_length = 56;      // Długość PCB
hole_dist_w = 58;     // Rozstaw otworów w szerokości
hole_dist_l = 49;     // Rozstaw otworów w długości
hole_diameter = 2.5;  // Średnica śruby (M2.5)

// --- Parametry ramy montażowej ---
frame_thickness = 5;  // Grubość ramy (szerokość ścianki)
frame_height = 5;     // Wysokość ramy/płyty base
post_height = 8;      // Całkowita wysokość tulei montażowych
post_diameter = 7;    // Zewnętrzna średnica tulei montażowej
strut_width = 5;      // Szerokość przekątnych wsporników

// Obliczenie offsetu dla otworów
offset_w = (rpi_width - hole_dist_w) / 2;
offset_l = (rpi_length - hole_dist_l) / 2;

// Wymiary wewnętrznego otworu ramy
inner_w = rpi_width - (frame_thickness * 2);
inner_l = rpi_length - (frame_thickness * 2);

module rpi_3a_frame() {
    
    right(case_margin_left) back(3)
      difference() {
        union() {
            translate([display_pin_1_margin_left - pin_r,16.25,12]) cube(
                [8, 3, display_height - 25],
                anchor=BACK+LEFT+BOT,
                center = false
            );
            
            translate([display_pin_3_margin_left - pin_r,16.25,12]) cube(
                [8, 3, display_height - 25],
                anchor=BACK+LEFT+BOT,
                center = false
            );
            
            translate([display_pin_1_margin_left + pin_r, 13.25, 12]) cube(
                [49.75, 6, frame_height]
            );
        };
    
        union() {
            translate([display_pin_1_margin_left, pin_deep+9.7 - 2, display_pin_1_margin_bottom + 22]) {
                ycyl(d=screw_insert_d, h=screw_h,center=false);
            }
            translate([display_pin_2_margin_left, pin_deep+9.7 - 2, display_pin_2_margin_bottom - 10]) {
                ycyl(d=screw_insert_d, h=screw_h,center=false);                
            }
            translate([display_pin_3_margin_left, pin_deep+9.7 - 2, display_pin_3_margin_bottom + 22]) {
                ycyl(d=screw_insert_d, h=screw_h,center=false);               
            }
            translate([display_pin_4_margin_left, pin_deep+9.7 - 2, display_pin_4_margin_bottom - 10]) {
                ycyl(d=screw_insert_d, h=screw_h,center=false);               
            }
        };
    };
    
    right(display_pin_1_margin_left + pin_hole_r - 0.5 + case_margin_extra) back(22.2) up(12) difference() {
        union() {
            // 1. Główna rama prostokątna (pusta w środku)
            difference() {
                cube([rpi_width, rpi_length, frame_height]);
                translate([frame_thickness, frame_thickness, -1])
                    cube([inner_w, inner_l, frame_height + 2]);
            }
            
            // 2. Przekątne wspierające (Krzyżak) wewnątrz ramy
            // Przesuwamy punkt startowy do wewnętrznego narożnika ramy
            translate([frame_thickness, frame_thickness, 0]) {
                linear_extrude(height = frame_height) {
                    // Pierwsza przekątna: od lewego dolnego do prawego górnego
                    hull() {
                        translate([0, 0]) circle(d = strut_width);
                        translate([inner_w, inner_l]) circle(d = strut_width);
                    }
                    // Druga przekątna: od lewego górnego do prawego dolnego
                    hull() {
                        translate([0, inner_l]) circle(d = strut_width);
                        translate([inner_w, 0]) circle(d = strut_width);
                    }
                }
            }
            
            // 3. Tuleje montażowe (słupki pod śruby)
            tuleje_pozycja() {
                cylinder(h = post_height, d = post_diameter);
            }
        }
        
        // 4. Otwory na śruby
        tuleje_pozycja() {
            translate([0, 0, -1])
                cylinder(h = post_height + 2, d = screw_insert_d);
        }
    }
}

// Pomocniczy moduł do pozycjonowania elementów w miejscach otworów RPi 3A+
module tuleje_pozycja() {
    translate([offset_w, offset_l, 0]) children();
    translate([offset_w + hole_dist_w, offset_l, 0]) children();
    translate([offset_w, offset_l + hole_dist_l, 0]) children();
    translate([offset_w + hole_dist_w, offset_l + hole_dist_l, 0]) children();
}

module bottom_case() {
    difference() {
        union() {
            translate([0, 19.25, 0]) rotate([11, 0, 0]) {
                cube([case_width, 63, 2]);   
            }
            
            translate([0, 16.2, 0]) {
                cube([2, 64.5, display_height]);
                translate([case_width - 2, 0, 0]) cube([2, 64.5, display_height]);
            }
            
            translate([0, 19.25 + 3, 0]) {
                translate([5 + mount_margin_left / 2, 0, 16]) rotate([90, 0, 0]) {
                    difference() {
                        cuboid([8 + mount_margin_left, 8, 4], anchor=[0, 0, -1], rounding=4, edges=[FWD+RIGHT, BACK+RIGHT]);
                        translate([mount_margin_left / 2, 0, 0]) cylinder(h = 5, d = hole_diameter);
                    }
                }
                translate([5 + mount_margin_left / 2, 0, 58]) rotate([90, 0, 0]) {
                    difference() {
                        cuboid([8 + mount_margin_left, 8, 4], anchor=[0, 0, -1], rounding=4, edges=[FWD+RIGHT, BACK+RIGHT]);
                        translate([mount_margin_left / 2, 0, 0]) cylinder(h = 5, d = hole_diameter);
                    }
                }
                translate([case_width - 5 - mount_margin_right / 2, 0, 16]) rotate([90, 0, 0]) {
                    difference() {
                        cuboid([8 + mount_margin_right, 8, 4], anchor=[0, 0, -1], rounding=4, edges=[FWD+LEFT, BACK+LEFT]);
                        translate([mount_margin_right / -2, 0, 0]) cylinder(h = 5, d = hole_diameter);
                    }
                }
                translate([case_width - 5 - mount_margin_right / 2, 0, 58]) rotate([90, 0, 0]) {
                    difference() {
                        cuboid([8 + mount_margin_right, 8, 4], anchor=[0, 0, -1], rounding=4, edges=[FWD+LEFT, BACK+LEFT]);
                        translate([mount_margin_right / -2, 0, 0]) cylinder(h = 5, d = hole_diameter);
                    }
                }
            }
            
            for (y = [0, 16, 36, 50.2]) {
                translate([1, 16 + 6 + y, 0]) difference() {
                    cuboid(
                        [6, 6, display_height - 2], 
                        anchor=[-1, -1, -1],
                        rounding=2, 
                        edges=[FWD+RIGHT,BACK+RIGHT]
                    );
                    translate([3.5, 3, display_height - 2 - 7.9]) cylinder(h = 8, d = screw_insert_d);
                } 
                
                translate([case_width - 6 - 1, 16 + 6 + y, 0]) difference() {
                    cuboid(
                        [6, 6, display_height - 2], 
                        anchor=[-1, -1, -1],
                        rounding=2, 
                        edges=[FWD+LEFT,BACK+LEFT]
                    );
                    translate([2.5, 3, display_height - 2 - 7.9]) cylinder(h = 8, d = screw_insert_d);
                }    
            }
            
            translate([0, 59.45 + 19.25, 0]) {
                translate([5 + mount_margin_right / 2, 0, 22]) rotate([90, 0, 0]) {
                    difference() {
                        translate([0, 3, 0]) cuboid([8 + mount_margin_right, 18, 4], anchor=[0, 1, -1], rounding=4, edges=[BACK+RIGHT]);
                        translate([mount_margin_right / 2, 0, 0]) cylinder(h = 5, d = screw_insert_d);
                    }
                }
                translate([5 + mount_margin_right / 2, 0, 53]) rotate([90, 0, 0]) {
                    difference() {
                        translate([0, 3.5, 0]) cuboid([8 + mount_margin_right, 14, 4], anchor=[0, 0, -1], rounding=4, edges=[FWD+RIGHT, BACK+RIGHT]);
                        translate([mount_margin_right / 2, 0, 0]) cylinder(h = 5, d = screw_insert_d);
                        translate([mount_margin_right / 2, 7, 0]) cylinder(h = 5, d = screw_insert_d);
                    }
                }
                translate([case_width - 5 - mount_margin_right / 2, 0, 22]) rotate([90, 0, 0]) {
                    difference() {
                        translate([0, 3, 0]) cuboid([8 + mount_margin_right, 18, 4], anchor=[0, 1, -1], rounding=4, edges=[BACK+LEFT]);
                        translate([mount_margin_right / -2, 0, 0]) cylinder(h = 5, d = screw_insert_d);
                    }
                }
                translate([case_width - 5 - mount_margin_right / 2, 0, 53]) rotate([90, 0, 0]) {
                    difference() {
                        translate([0, 3.5, 0]) cuboid([8 + mount_margin_right, 14, 4], anchor=[0, 0, -1], rounding=4, edges=[FWD+LEFT, BACK+LEFT]);
                        translate([mount_margin_right / -2, 0, 0]) cylinder(h = 5, d = screw_insert_d);
                        translate([mount_margin_right / -2, 7, 0]) cylinder(h = 5, d = screw_insert_d);
                    }
                }
            }
            
            translate([2, 59.45 + 19.25, 5])
                    //    cube([case_width - 4, 2, 49.5 + 2]);
                cube([case_width - 4, 2, 10 + 2]);
            
            translate([2, 59.45, 5.5])
                    //    cube([case_width - 4, 2, 49.5 + 2]);
                cuboid([case_width - 4, 2, 10 + 1], anchor=[-1, 0, 0], rounding=1, edges=[FWD+TOP, BACK+TOP]);
            translate([2, 59.45 - 18, 2.5])
                    //    cube([case_width - 4, 2, 49.5 + 2]);
                cuboid([case_width - 4, 2, 10 + 1], anchor=[-1, 0, 0], rounding=1, edges=[FWD+TOP, BACK+TOP]);
            translate([2, 59.45 - 32, 0])
                    //    cube([case_width - 4, 2, 49.5 + 2]);
                cuboid([case_width - 4, 2, 10 + 1], anchor=[-1, 0, 0], rounding=1, edges=[FWD+TOP, BACK+TOP]);
        }
        
        translate([-10, 19.25, -30.56]) rotate([11, 0, 0]) {
            cube([case_width + 20, 73, 30]);   
        }
        
        cube([case_width, 19.25, display_height]);
                
        rpi_3a_frame();
        
        translate([case_margin_extra, 0, 0]) right(display_pin_1_margin_left + pin_hole_r - 0.5) back(22.2) up(12)
                cube([rpi_width, rpi_length, frame_height]);
    }
    
    difference() {
        union() {
            translate([0, 16.3, 2])
                cube([35.4, 6, 2]);
            translate([35.4 + 8, 16.3, 2])
                cube([49.75, 6, 2]);
            translate([35.4 + 49.75 + 8 + 8, 16.3, 2])
                cube([20, 6, 2]);
        }
        
        front_case();
    }
    
    translate([2,16,20])
        cube([2, 6, 34]);
    translate([case_width - 2 - 2,16,20])
        cube([2, 6, 34]);
}

module switch_socket_cutoff(unit_size, height, bottom_space_height) {
    plate_hole_size = 14.0;
    plate_weight = 1.2;
    switch_deep = 2.2;

    center_post_h = 5.6;
    center_post_d = 5.1;

    pin_weight = 1.8;
    pin_height = 2.9;

    box_hole_width = 14.0;
    box_hole_height = 15.0;
    box_weight = height - plate_weight;

    space_on_top_height = 10;

    pin_1_position = [2.6, -5.75, 0];
    pin_2_position = [-4.7, -4.4, 0];

    translate([unit_size / 2, unit_size / 2, plate_weight]) {
        // switch simulation
        translate([0, 0, -switch_deep])
            cube([plate_hole_size, plate_hole_size, switch_deep], center=true);

        // plate
        translate([0, 0, plate_weight / 2])
            cube([plate_hole_size, plate_hole_size, plate_weight + tol], center=true);

        // box
        translate([0, 0, -box_weight / 2])
            cube([box_hole_height, box_hole_width, box_weight + tol], center=true);
    };

    translate([0, 0, height - tol])
        cube([unit_size + tol, unit_size + tol, space_on_top_height]);

    translate([-tol, -tol, -bottom_space_height - tol])
        cube([unit_size + tol2, unit_size + tol2, bottom_space_height + tol2]);

    translate([unit_size / 2, unit_size / 2, -center_post_h])
        cylinder(h=center_post_h, d=center_post_d);

    translate([unit_size / 2, unit_size / 2, -pin_height]) {
        // pin 1
        translate(pin_1_position)
            cylinder(h = pin_height, d = pin_weight);

        // pin 2
        translate(pin_2_position)
            cylinder(h = pin_height, d = pin_weight);
    }
}

module top_case() {
    unit_size = 18.5;
    unit_count = 4;
    translate([2, 19.25, display_height - 2]) difference() {
        union() {
            difference() {
                union() {
                    cube([case_width - 4, 21, 2]);
                    
                    translate([(case_width - 4 - unit_size * unit_count) / 2 - 2, 2, -4.79 - 3.7]) 
                        cube([unit_size * unit_count + 4, unit_size + 4, 0 + 6 + 3.7]);
    
                    translate([0, 21, 0])
                        cube([case_width - 4, 38.95, 2]);
                    
                    translate([0, 10.5, 0])
                        cuboid([case_width - 4,2,4], anchor=[-1,-1,1], rounding=1, edges=[FWD+DOWN,BACK+DOWN]);
                    translate([0, 32.5, 0])
                        cuboid([case_width - 4,2,2], anchor=[-1,-1,1], rounding=1, edges=[FWD+DOWN,BACK+DOWN]);
                    translate([0, 46.5, 0])
                        cuboid([case_width - 4,2,2], anchor=[-1,-1,1], rounding=1, edges=[FWD+DOWN,BACK+DOWN]);
                    
                    //translate([0, 59.45, -49.5])
                    //    cube([case_width - 4, 2, 49.5 + 2]);
                    translate([0, 59.45, -7])
                        cube([case_width - 4, 2, 6 + 3]);
                    
                    translate([23, 0, -1]) 
                        cuboid([19, 6, 2]);
                    translate([66.25, 0, -1]) 
                        cuboid([48, 6, 2]);
                    translate([104, 0, -1]) 
                        cuboid([8, 6, 2]);
                }
                
                translate([0, 59.45 + 2, -3.5]) {
                    translate([5 + mount_margin_right / 2 - 2, 0, 0]) rotate([90, 0, 0]) {
                        translate([mount_margin_right / 2, 0, 0]) {
                            cylinder(h = 5, d = hole_diameter);
                            cylinder(h = 1, d = hole_diameter + 2);
                        }
                    }
                    translate([case_width - 5 - mount_margin_right / 2 - 2, 0, 0]) rotate([90, 0, 0]) {
                        translate([mount_margin_right / -2, 0, 0]) {
                            cylinder(h = 5, d = hole_diameter);
                            cylinder(h = 1, d = hole_diameter + 2);
                        }
                    }
                }
                
                /*for (y = [0, 50.2]) {
                    translate([2.5, 3 + 2.75 + y, -0.1]) {
                        cylinder(h = 2.2, d = hole_diameter);
                        translate([0, 0, 1.2]) cylinder(h = 1, d = hole_diameter + 1.5);
                    }
                    translate([case_width - 6 - 0.5, 3 + 2.75 + y, -0.1]){
                        cylinder(h = 2.2, d = hole_diameter);
                        translate([0, 0, 1.2]) cylinder(h = 1, d = hole_diameter + 1.5);
                    }
                }*/
                        
                /*translate([0, 59.45 + 1.2, -9]) for (y = [0]) {
                    rotate([90, 0, 0]) translate([2.5, 3 + 2.75 + y, -0.1]) {
                        cylinder(h = 2.2, d = hole_diameter);
                        translate([0, 0, -0.9]) cylinder(h = 1, d = hole_diameter + 1.5);
                    }
                    rotate([90, 0, 0]) translate([case_width - 6 - 0.5, 3 + 2.75 + y, -0.1]){
                        cylinder(h = 2.2, d = hole_diameter);
                        translate([0, 0, -0.9]) cylinder(h = 1, d = hole_diameter + 1.5);
                    }
                }*/
                
                translate([(case_width - 4 - unit_size * unit_count) / 2, 4, 0]) {
                    for (i = [0:1:unit_count-1])
                        translate([i * unit_size, 0, -6 - 3.7]) switch_socket_cutoff(
                            unit_size = unit_size,
                            height = 2.4, //@todo fix for another height!
                            bottom_space_height = 3.5
                        );
                }
                
                //translate([(case_width - 4 - 4 - unit_size * unit_count) / 2, unit_size + 4, 0]) {
                //    cube([unit_count * unit_size + 4, 38, 2]);
                //}
                
                translate([(case_width - 4 - unit_size * unit_count) / 2 - 0.5, unit_size + 8, 0])
                    for(i = [0:2:unit_size * unit_count])
                        translate([i, 0, 0]) cube([1, 30, 2]);
                     
            }

            /*translate([(case_width - 4 - 4 - unit_size * unit_count) / 2, unit_size + 4, 0]) linear_extrude(height = 2) {
                // Zależnie od tego, gdzie jest środek układu współrzędnych w SVG
                // może zajść potrzeba przesunięcia (translate) wzoru na środek
                resize([unit_count * unit_size + 4, 38, 2])
                rotate([180, 0, 90])
                import("./surface1.svg"); 
            }*/
        }
    }
}

module back_case() {
    difference() {
        translate([2, 59.45 + 19.25, 17])
            cube([case_width - 4, 2, 41]);
        
        translate([0, 59.45 + 19.25 + 2, 0]) {
            translate([5 + mount_margin_right / 2, 0, 22]) rotate([90, 0, 0]) {
                translate([mount_margin_right / 2, 0, 0]) {
                    cylinder(h = 5, d = hole_diameter);
                    cylinder(h = 1, d = hole_diameter + 2);
                }
            }
            translate([5 + mount_margin_right / 2, 0, 53]) rotate([90, 0, 0]) {
                translate([mount_margin_right / 2, 0, 0]) {
                    cylinder(h = 5, d = hole_diameter);
                    cylinder(h = 1, d = hole_diameter + 2);
                }
            }
            translate([case_width - 5 - mount_margin_right / 2, 0, 22]) rotate([90, 0, 0]) {
                translate([mount_margin_right / -2, 0, 0]) {
                    cylinder(h = 5, d = hole_diameter);
                    cylinder(h = 1, d = hole_diameter + 2);
                }
            }
            translate([case_width - 5 - mount_margin_right / 2, 0, 53]) rotate([90, 0, 0]) {
                translate([mount_margin_right / -2, 0, 0]) {
                    cylinder(h = 5, d = hole_diameter);
                    cylinder(h = 1, d = hole_diameter + 2);
                }
            }
        }
    }
}

// @TODO otwory na ekran za małe
// @TODO mocowanie ekranu, za głębokie, przerobić na gwinty

module front_glass() {
    glass_weight = 3;
    glass_screw_d = 3.3;
    
    difference() {
        translate([0, glass_weight * -1, 0]) cuboid(
            [display_width + case_margin_left + case_margin_right, display_height, glass_weight],
            anchor=BACK+LEFT+BOT,
            orient=BACK
        );

        for (x = [0, 1]) for(z = [0, 1]) {
            translate([3.6 + (x * (case_width - (3.6 * 2))), 0, 3.6 + (z * (display_height - (3.6 * 2)))]) {
                rotate([90,0,0]) cylinder(h=9, d=glass_screw_d);
            }
        }
    }
}

module case() {
    //projection(cut = false) rotate([90, 0, 0]) front_glass();
    //    front_glass();
    
    //front_case();
    
    //rpi_3a_frame();
    
    //bottom_case();
    
    top_case();
    
    //back_case();
}

case();
