# Keyberon grid [![Build status](https://travis-ci.org/TeXitoi/keyberon-grid.svg?branch=master)](https://travis-ci.org/TeXitoi/keyberon)

A hand wired ortholinear mechanical keyboard with a firmware in pure
Rust. The case uses a parametric design allowing to create a grid
keyboard of any size. The firmware allows you to customize each key as
you wish: A layer change (as the function key), a key combo (as one
key for the paste shortcut) or a regular key.

![photo](images/keyberon.jpg)

![photo](images/keyberon5x15.jpg)

You can [build](BUILDING.md) this keyboard yourself quite easily.

## The case

The [OpenSCad files](cad/) are a totally parametric design. You can generate a grid of keys of any size by modifying the parameters. It is designed to be as low as possible.

## The firmware

The firmware, [Keyberon](https://github.com/TeXitoi/keyberon), is
written in the [rust programming language](https://rust-lang.org).

# FAQ

## I want to use your 3D printed case, but I want to use a rock solid firmware

[QMK](https://github.com/qmk/qmk_firmware) supports the blue pill board. You may have to search a bit, but it should not be so complicated. See for example the [BluePill handwired](https://github.com/qmk/qmk_firmware/tree/master/keyboards/handwired/bluepill). Maybe the [firmwares from Cannon Keys](https://github.com/qmk/qmk_firmware/tree/master/keyboards/cannonkeys) can also be interesting. If you do any progress on this side, feel free to open a PR to share your experience.

## Keyberon, what's that name?

To find new, findable and memorable project names, some persons in the rust community try to mix the name of a city with some keyword related to the project. For example, you have the [Tokio project](https://tokio.rs/) that derive its name from the Japanese capital Tokyo and IO for Input Output, the main subject of this project.

So, I have to find such a name. In the mechanical keyboard community, "keeb" is slang for keyboard. Thus, I searched for a city with the sound [kib], preferably in France as it is the country of origin of the project. I found [Quiberon](https://en.wikipedia.org/wiki/Quiberon), and thus I named the project Keyberon.

## What is this black and white pattern on your keyboard?

I'm fascinated by [isomorphic keyboards](https://en.wikipedia.org/wiki/Isomorphic_keyboard). Thus, I've searched a bit which vectors can do a great isomorphic keyboard on a 12x5 grid. After some trial and errors, I found that a major third left, and a minor third up is promising. Then, searching for an origin, I found one that do a great symmetry on the home row using the colors of the piano keys.

So, the black keys on the home row are G#, the whites at left of G# are C, and the whites above G# are B, and so on.

My keyboard doesn't (yet) play any music, that's purely aesthetics.

## What's the layout

As an old user of the [TypeMatrix 2030](http://www.typematrix.com/2030/features.php), the layout is quite close of the layout of the TypeMatrix. I also maximize the use of the thumbs by having shift, space, enter, alt, alt gr, gui and backspace on the thumbs.

Layer 0:
```
┌────┬────┬────┬────┬────┬────╥────┬────┬────┬────┬────┬────┐
│ ~  │ !  │ @  │ #  │ $  │ %  ║ ^  │ &  │ *  │ (  │ )  │ _  │
│ `  │ 1  │ 2  │ 3  │ 4  │ 5  ║ 6  │ 7  │ 8  │ 9  │ 0  │ -  │
├────┼────┼────┼────┼────┼────╫────┼────┼────┼────┼────┼────┤
│ ↹  │ Q  │ W  │ E  │ R  │ T  ║ Y  │ U  │ I  │ O  │ P  │ {  │
│    │    │    │    │    │    ║    │    │    │    │    │ [  │
├────┼────┼────┼────┼────┼────╫────┼────┼────┼────┼────┼────┤
│ }  │ A  │ S  │ D  │ F  │ G  ║ H  │ J  │ K  │ L  │ :  │ "  │
│ ]  │    │    │    │    │    ║    │    │    │    │ ;  │ '  │
├────┼────┼────┼────┼─══─┼────╫────┼─══─┼────┼────┼────┼────┤
│ +  │ Z  │ X  │ C  │ V  │ B  ║ N  │ M  │ <  │ >  │ ?  │ |  │
│ =  │    │    │    │    │    ║    │    │ ,  │ .  │ /  │ \  │
├────┼────┼────┼────┼────┼────╫────┼────┼────┼────┼────┼────┤
│    │    │ GUI│ Alt│ ␣/ │ ⇧  ║ ⇧  │ ⏎/ │ Alt│ ⌫  │    │    │
│    │    │    │    │L(1)│    ║    │Ctrl│    │    │    │    │
└────┴────┴────┴────┴────┴────╨────┴────┴────┴────┴────┴────┘
```
Legend:
 - L(1): layer 1 when pressed
 - A/B: A when tapped, B when hold

Layer 1:
```
┌────┬────┬────┬────┬────┬────╥────┬────┬────┬────┬────┬────┐
│ F1 │ F2 │ F3 │ F4 │ F5 │ F6 ║ F7 │ F8 │ F9 │ F10│ F11│ F12│
│    │    │    │    │    │    ║    │    │    │    │    │    │
├────┼────┼────┼────┼────┼────╫────┼────┼────┼────┼────┼────┤
│    │Paus│    │Pr. │    │    ║    │    │Del.│    │    │    │
│    │    │    │Scr.│    │    ║    │    │    │    │    │    │
├────┼────┼────┼────┼────┼────╫────┼────┼────┼────┼────┼────┤
│    │    │Num │Ins.│Esc.│    ║ ⇪  │ ◄  │ ▼  │ ▲  │ ►  │    │
│    │    │Lock│    │    │    ║    │    │    │    │    │    │
├────┼────┼────┼────┼─══─┼────╫────┼─══─┼────┼────┼────┼────┤
│    │Undo│ Cut│Copy│Past│    ║    │ ⇱  │ ⇟  │ ⇞  │ ⇲  │    │
│    │    │    │    │    │    ║    │    │    │    │    │    │
├────┼────┼────┼────┼────┼────╫────┼────┼────┼────┼────┼────┤
│    │    │    │    │    │    ║    │C-⏎ │    │    │    │    │
│    │    │    │    │    │    ║    │    │    │    │    │    │
└────┴────┴────┴────┴────┴────╨────┴────┴────┴────┴────┴────┘
```
Legend:
 - C-⏎: Control+Enter (at the same time)


I use the [bépo layout](https://bepo.fr), so this is what I have when I type:
```
┌────┬────┬────┬────┬────┬────╥────┬────┬────┬────┬────┬────┐
│ #  │ 1  │ 2  │ 3  │ 4  │ 5  ║ 6  │ 7  │ 8  │ 9  │ 0  │ °  │
│ $  │ " —│ « <│ » >│ ( [│ ) ]║ @ ^│ +  │ -  │ /  │ *  │ =  │
├────┼────┼────┼────┼────┼────╫────┼────┼────┼────┼────┼────┤
│ ↹  │ B  │ É  │ P  │ O  │ È  ║ !  │ V  │ D  │ L  │ J  │ Z  │
│    │   |│    │   &│   œ│    ║ ^  │    │    │    │    │    │
├────┼────┼────┼────┼────┼────╫────┼────┼────┼────┼────┼────┤
│ W  │ A  │ U  │ I  │ E  │ ;  ║ C  │ T  │ S  │ R  │ N  │ M  │
│    │   æ│   ù│   ¨│   €│ ,  ║    │    │    │    │    │    │
├────┼────┼────┼────┼─══─┼────╫────┼─══─┼────┼────┼────┼────┤
│ `  │ À  │ Y  │ X  │ :  │ K  ║ ?  │ Q  │ G  │ H  │ F  │ Ç  │
│ %  │   \│   {│   }│ . …│   ~║ '  │    │    │    │    │    │
├────┼────┼────┼────┼────┼────╫────┼────┼────┼────┼────┼────┤
│    │    │ GUI│ Alt│nbsp│ ⇧  ║ ⇧  │ ⏎  │Alt │ ⌫  │    │    │
│    │    │    │    │ ␣ _│    ║    │    │  Gr│    │    │    │
└────┴────┴────┴────┴────┴────╨────┴────┴────┴────┴────┴────┘
```

You can of course tune the layout as you wish easily.
