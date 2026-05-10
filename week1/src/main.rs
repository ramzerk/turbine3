use rand;
use std::{cmp::Ordering, io};

fn main() {
    println!("Guess the number");
    let secret_number = rand::random_range(1..=100);
    loop {
        println!("Please input your guess.");

        let mut guess = String::new();

        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        let guess: u32 = match guess.trim().parse() {
            Ok(num) => {
                println!("You guessed: {guess}");
                num
            }
            Err(_) => {
                println!("Input a number please");
                continue
            }
        };
        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too Small"),
            Ordering::Greater => println!("Too Big"),
            Ordering::Equal => {
                println!("You win");
                break;
            }
        }
    }
}
