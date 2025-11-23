include <BOSL2/std.scad>

$f=128;

display_width=104.37;
display_height=65.44;
display_rounding=4;

przekatna = sqrt(display_width * display_width + display_height * display_height);

translate([0, 0, -5.6]) translate([0, 100, 39.5]) rotate([82,0,0]) {
    rect_tube(size = [display_width+2, display_height+2], h = 80, wall=3, rounding=display_rounding);
    
    rotate([0, 0, 45]) cube([przekatna, 2, 5], center=true);
}
