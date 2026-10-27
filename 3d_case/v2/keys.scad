include <BOSL2/std.scad>
tol = 0.001;
tol2 = tol * 2;
unit_size = 18.5;

$fn = 64;

// --- Parametry Główne ---
width = 18.0;         // Szerokość keycapa (1U to zazwyczaj ok. 18mm)
depth = 18.0;         // Długość keycapa
top_thickness = 2;  // Grubość górnej, płaskiej ścianki
stem_height = 3.6;    // Wysokość trzpienia (zapewnia dystans od obudowy switcha)
corner_radius = 0.5;  // Zaokrąglenie rogów

// --- Tolerancje krzyżaka (Wymagają kalibracji pod konkretną drukarkę FDM) ---
stem_outer_d = 5.5;   // Zewnętrzna średnica cylindra trzpienia
cross_length = 4.3;   // Długość wycięcia (standard MX to ok. 4.0mm)
cross_width = 1.25;   // Grubość wycięcia (standard MX to ok. 1.17mm)

module flat_keycap() {
    union() {
        // Główna, płaska płytka (Top)
        hull() {
            translate([-width/2+corner_radius, -depth/2+corner_radius, 0])
                cylinder(r=corner_radius, h=top_thickness);
            translate([width/2-corner_radius, -depth/2+corner_radius, 0])
                cylinder(r=corner_radius, h=top_thickness);
            translate([-width/2+corner_radius, depth/2-corner_radius, 0])
                cylinder(r=corner_radius, h=top_thickness);
            translate([width/2-corner_radius, depth/2-corner_radius, 0])
                cylinder(r=corner_radius, h=top_thickness);
        }
        
        // Trzpień na switch MX (Stem)
        translate([0, 0, top_thickness])
        difference() {
            // Zewnętrzny cylinder
            cylinder(d=stem_outer_d, h=stem_height);

            // Wycięcie na krzyżak
            translate([0, 0, -0.01]) {
                translate([-cross_width/2, -cross_length/2, 0])
                    cube([cross_width, cross_length, stem_height + 0.1]);
                translate([-cross_length/2, -cross_width/2, 0])
                    cube([cross_length, cross_width, stem_height + 0.1]);
            }
        }
    }
}

flat_keycap();