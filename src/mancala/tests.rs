#![cfg(test)]

use super::*;

#[test]
fn play_in_bounds() {
    for i in 0..2 {
        let j = 1 - i;

        let mut game = Game::default();

        let player = game.play(i, 0);

        assert_eq!(player, j);
        assert_eq!(game.boards[j], Board([4, 4, 4, 4, 4, 4]));
        assert_eq!(game.boards[i], Board([0, 5, 5, 5, 5, 4]));

        assert_eq!(game.points[j], 0);
        assert_eq!(game.points[i], 0);
    }
}

#[test]
fn play_combo() {
    for i in 0..2 {
        let j = 1 - i;

        let mut game = Game::default();

        let player = game.play(i, 2);

        assert_eq!(player, i);
        assert_eq!(game.boards[j], Board([4, 4, 4, 4, 4, 4]));
        assert_eq!(game.boards[i], Board([4, 4, 0, 5, 5, 5]));

        assert_eq!(game.points[j], 0);
        assert_eq!(game.points[i], 1);
    }
}

#[test]
fn play_into_opposing() {
    for i in 0..2 {
        let j = 1 - i;

        let mut game = Game::default();

        let player = game.play(i, 3);

        assert_eq!(player, j);
        assert_eq!(game.boards[j], Board([5, 4, 4, 4, 4, 4]));
        assert_eq!(game.boards[i], Board([4, 4, 4, 0, 5, 5]));

        assert_eq!(game.points[j], 0);
        assert_eq!(game.points[i], 1);
    }
}

#[test]
fn play_go_full_circle() {
    for i in 0..2 {
        let j = 1 - i;

        let mut game = Game::default();

        game.boards[i].0[5] = 8;

        let player = game.play(i, 5);

        assert_eq!(player, j);
        assert_eq!(game.boards[j], Board([5, 5, 5, 5, 5, 5]));
        assert_eq!(game.boards[i], Board([5, 4, 4, 4, 4, 0]));

        assert_eq!(game.points[j], 0);
        assert_eq!(game.points[i], 1);
    }
}

#[test]
fn play_in_opposing() {
    for i in 0..2 {
        let j = 1 - i;

        let mut game = Game::default();
        game.boards[i].0 = [0; 6];

        let player = game.play(i, 6);

        assert_eq!(player, j);
        assert_eq!(game.boards[j], Board([0, 5, 5, 5, 5, 4]));
        assert_eq!(game.boards[i], Board([0, 0, 0, 0, 0, 0]));

        assert_eq!(game.points[j], 0);
        assert_eq!(game.points[i], 0);
    }
}

#[test]
fn play_in_opposing_and_loop() {
    for i in 0..2 {
        let j = 1 - i;

        let mut game = Game::default();
        game.boards[i].0 = [0; 6];

        let player = game.play(i, 11);

        assert_eq!(player, j);
        assert_eq!(game.boards[j], Board([4, 4, 0, 4, 4, 0]));
        assert_eq!(game.boards[i], Board([1, 1, 1, 5, 0, 0]));

        assert_eq!(game.points[j], 0);
        assert_eq!(game.points[i], 0);
    }
}

#[test]
#[should_panic]
fn play_when_no_stones() {
    for i in 0..2 {
        let mut game = Game::default();
        game.boards[i].0 = [0; 6];

        game.play(i, 0);
    }
}

#[test]
#[should_panic]
fn play_on_stoneless_cell() {
    for i in 0..2 {
        let mut game = Game::default();
        game.boards[i].0[0] = 0;

        game.play(i, 0);
    }
}

#[test]
fn take_stone() {
    for i in 0..2 {
        let j = 1 - i;

        let mut game = Game::default();
        game.boards[i].0[5] = 0;

        let player = game.play(i, 1);

        assert_eq!(player, j);
        assert_eq!(game.boards[j], Board([0, 4, 4, 4, 4, 4]));
        assert_eq!(game.boards[i], Board([4, 0, 5, 5, 5, 5]));

        assert_eq!(game.points[j], 0);
        assert_eq!(game.points[i], 0);
    }
}

#[test]
fn dont_take_stone() {
    for i in 0..2 {
        let j = 1 - i;

        let mut game = Game::default();
        game.boards[j].0[0] = 0;

        let player = game.play(i, 3);

        assert_eq!(player, j);
        assert_eq!(game.boards[j], Board([1, 4, 4, 4, 4, 4]));
        assert_eq!(game.boards[i], Board([4, 4, 4, 0, 5, 5]));

        assert_eq!(game.points[j], 0);
        assert_eq!(game.points[i], 1);
    }
}
