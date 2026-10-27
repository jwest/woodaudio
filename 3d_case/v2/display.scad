include <BOSL2/std.scad>
$fn=64;

display_width = 104.35;
display_height = 65.5;
display_weight = 1.3;
display_radious = 4.0;

display_plate_margin = 3;
//display_plate_margin_left = 1.5;
display_plate_width = display_width - 2 * display_plate_margin;
display_plate_height = display_height - 2 * display_plate_margin;
display_plate_weight = 6.7;

display_pin_r = 5.5 / 2;
display_pin_h = 7.8;

display_pin_margin_top = 5.7;
display_pin_margin_bottom = 4.8;
display_pin_margin_right = 25.3;
display_pin_margin_left = 15.3;

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

display();