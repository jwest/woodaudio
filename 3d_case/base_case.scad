/*
 * Parametryczny moduł gniazda (socketu) dla przełącznika typu MX
 * Wersja z pełnym dnem i otworami na piny (standard 5-pin / PCB Mount).
 *
 * Orientacja:
 * Oś X: Wzdłuż klipsów mocujących (wymiar 15.8mm).
 * Oś Y: Wzdłuż boków 14mm.
 * Oś Z: Wysokość.
 */
include <BOSL2/std.scad>

// Ustawienie domyślnej rozdzielczości
$fn = 128;

base_width = 104.4;
display_width=104.37;
display_height=65.44;
display_rounding=4;

difference() {

    union() {
        
        difference() {
            translate([0, 0, -2.8]) hull() {
                rect_tube(
                    size=[base_width+10, 24], 
                    isize=[base_width-14, 18],
                    h=5.5,
                    rounding=[0,0,5,5],
                    irounding=0
                );
                translate([0, -6, +3]) rotate([0, 90, 0]) cylinder(h=base_width, r=7, center=true);
                translate([base_width / -2, 12, 10]) rotate([90, 0, 0]) cylinder(h=20, r=3);
                translate([base_width / 2, 12, 10]) rotate([90, 0, 0]) cylinder(h=20, r=3);
                translate([base_width / -2, 12, -1]) rotate([90, 0, 0]) cylinder(h=20, r=3);
                translate([base_width / 2, 12, -1]) rotate([90, 0, 0]) cylinder(h=20, r=3);
            }
            translate([0, 0, 12.7]) cube([base_width - 5, 24.1, 20], center = true);
            translate([0, 1, 0]) cube([base_width - 14, 18, 5.5], center = true);
            translate([0, 3.1, -3.9]) cube([20.5, 20, 2.5], center = true);
            translate([0, 3.1, -3.9]) cube([60.5, 7, 2.5], center = true);
            translate([33, 2.1, -3.9]) cube([19.5, 18, 2.5], center = true);
            translate([-33, 2.1, -3.9]) cube([19.5, 18, 2.5], center = true);
            
            translate([0,-12.5,0]) cube([base_width - 5, 5, 13], center = true);
        }


        translate([0, 0, -5.6]) difference() {
            union() {
                translate([0, 0, 2.8]) hull() {
                        translate([0,57,0]) rect_tube(
                            size=[base_width+10, 90], 
                            isize=[base_width-14, 20],
                            h=5.5,
                            rounding=[5,5,0,0],
                            irounding=0
                        );
                        translate([0, 50, 0]) rotate([0, 90, 0]) cylinder(h=base_width, r=3, center=true);
                        translate([base_width / -2, 22, 10]) rotate([90, 0, 0]) cylinder(h=10, r=3);
                        translate([base_width / 2, 22, 10]) rotate([90, 0, 0]) cylinder(h=10, r=3);
                        translate([base_width / -2, 98, -1]) rotate([90, 0, 0]) cylinder(h=86, r=3);
                        translate([base_width / 2, 98, -1]) rotate([90, 0, 0]) cylinder(h=86, r=3);
                }
            };

            hull() {
                    rect_tube(
                        size=[base_width+10, 24], 
                        isize=[base_width-14, 20],
                        h=5.5,
                        rounding=[0,0,5,5],
                        irounding=0
                    );
                    translate([0, -6, +3]) rotate([0, 90, 0]) cylinder(h=base_width, r=7, center=true);
                    translate([base_width / -2, 12, 10]) rotate([90, 0, 0]) cylinder(h=20, r=3);
                    translate([base_width / 2, 12, 10]) rotate([90, 0, 0]) cylinder(h=20, r=3);
                    translate([base_width / -2, 12, -1]) rotate([90, 0, 0]) cylinder(h=20, r=3);
                    translate([base_width / 2, 12, -1]) rotate([90, 0, 0]) cylinder(h=20, r=3);
            }
            
            translate([0, 100, 39.5]) rotate([82,0,0]) rect_tube(size = [display_width+2, display_height+2], h = 80, wall=25, rounding=display_rounding);
            
        }

    }

    translate([0, 48.1, -3.9]) cube([40.5, 75, 2.5], center = true);
//    translate([35, 30.1, -3.9]) cube([20.5, 40, 2.5], center = true);
//    translate([-35, 30.1, -3.9]) cube([20.5, 40, 2.5], center = true);


    translate([0, 43.1, 0]) cube([40.5, 100, 5.5], center = true);
    translate([0, 60.1, 7.5]) cube([40.5, 45, 10.6], center = true);
//    translate([35, 33.1, 0]) cube([20.5, 80, 5.5], center = true);
//    translate([-35, 33.1, 0]) cube([20.5, 80, 5.5], center = true);
}

//translate([0, 0, -5.6]) translate([0, 100, 39.5]) rotate([82,0,0]) rect_tube(size = [display_width+2, display_height+2], h = 80, wall=3, rounding=display_rounding);