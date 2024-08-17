use crossterm::{
    cursor,
    event::{self, KeyCode, KeyEvent},
    terminal::{Clear, ClearType},
    ExecutableCommand,
    style::{Color, SetForegroundColor},
};
use rand::Rng;
use std::{
    collections::VecDeque,
    fs::OpenOptions,
    io::{self, Write},
    thread::sleep,
    time::Duration,
};

// Estrutura do jogo
#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct SnakeGame {
    snake: VecDeque<(usize, usize)>,
    direction: Direction,
    food: (usize, usize),
    width: usize,
    height: usize,
    game_over: bool,
    score: usize,
    buffer: Vec<Vec<char>>, // Buffer para armazenar o estado do terminal
}

impl SnakeGame {
    fn new(width: usize, height: usize) -> Self {
        let mut snake = VecDeque::new();
        snake.push_back((width / 2, height / 2)); // Snake starts in the middle
        let mut buffer = vec![vec![' '; width]; height];
        Self {
            snake,
            direction: Direction::Right,
            food: (rand::thread_rng().gen_range(1..width - 1), rand::thread_rng().gen_range(1..height - 1)),
            width,
            height,
            game_over: false,
            score: 0,
            buffer,
        }
    }

    fn update_buffer(&mut self) {
        // Limpa o buffer
        self.buffer.iter_mut().for_each(|row| row.fill(' '));

        // Adiciona a cobra no buffer
        for &(x, y) in &self.snake {
            if x < self.width && y < self.height {
                self.buffer[y][x] = '*';
            }
        }

        // Adiciona a comida no buffer
        if self.food.0 < self.width && self.food.1 < self.height {
            self.buffer[self.food.1][self.food.0] = '@';
        }

        // Adiciona as paredes no buffer
        for x in 0..self.width {
            self.buffer[0][x] = '#';
            self.buffer[self.height - 1][x] = '#';
        }
        for y in 0..self.height {
            self.buffer[y][0] = '#';
            self.buffer[y][self.width - 1] = '#';
        }
    }

    fn get_snake_color(&self) -> Color {
        match self.score {
            0..=4 => Color::Green,
            5..=9 => Color::Cyan,
            10..=14 => Color::Yellow,
            15..=19 => Color::Magenta,
            20..=24 => Color::Blue,
            25..=29 => Color::Red,
            _ => Color::AnsiValue(202), // Laranja (Ansi)
        }
    }

    fn draw(&self) {
        let mut stdout = io::stdout();
        stdout.execute(cursor::MoveTo(0, 0)).unwrap();

        for y in 0..self.height {
            for x in 0..self.width {
                let ch = self.buffer[y][x];
                let color = match ch {
                    '*' => self.get_snake_color(),
                    '@' => Color::Red,
                    '#' => Color::AnsiValue(94), // Marrom (Ansi)
                    _ => Color::Reset,
                };
                stdout.execute(SetForegroundColor(color)).unwrap();
                print!("{}", ch);
            }
            println!();
        }

        stdout.execute(SetForegroundColor(Color::Reset)).unwrap(); // Resetar cor para o texto de pontuação
        println!("Score: {}", self.score);
        stdout.flush().unwrap();
    }

    fn update(&mut self) {
        let head = self.snake.front().unwrap();
        let new_head = match self.direction {
            Direction::Up => (head.0, head.1.wrapping_sub(1)),
            Direction::Down => (head.0, head.1.wrapping_add(1)),
            Direction::Left => (head.0.wrapping_sub(1), head.1),
            Direction::Right => (head.0.wrapping_add(1), head.1),
        };

        if self.snake.contains(&new_head) || new_head.0 == 0 || new_head.1 == 0 || new_head.0 >= self.width - 1 || new_head.1 >= self.height - 1 {
            self.game_over = true;
            return;
        }

        self.snake.push_front(new_head);
        if new_head == self.food {
            self.score += 1;
            self.food = (rand::thread_rng().gen_range(1..self.width - 1), rand::thread_rng().gen_range(1..self.height - 1));
        } else {
            self.snake.pop_back();
        }
    }

    fn change_direction(&mut self, direction: Direction) {
        match (self.direction, direction) {
            (Direction::Up, Direction::Down) | (Direction::Down, Direction::Up) => {}
            (Direction::Left, Direction::Right) | (Direction::Right, Direction::Left) => {}
            _ => self.direction = direction,
        }
    }

    fn handle_input(&mut self) {
        if event::poll(Duration::from_millis(100)).unwrap() {
            if let event::Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                match code {
                    KeyCode::Esc => self.game_over = true,
                    KeyCode::Up => self.change_direction(Direction::Up),
                    KeyCode::Down => self.change_direction(Direction::Down),
                    KeyCode::Left => self.change_direction(Direction::Left),
                    KeyCode::Right => self.change_direction(Direction::Right),
                    _ => {}
                }
            }
        }
    }

    fn save_score(&self, name: Option<&str>) {
        let file_path = "scores.txt";
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .unwrap();
        if let Some(name) = name {
            writeln!(file, "{}: {}", name, self.score).unwrap();
        } else {
            writeln!(file, "Anonymous: {}", self.score).unwrap();
        }
    }

    fn display_scores() {
        let file_path = "scores.txt";
        let mut scores = vec![];

        if let Ok(contents) = std::fs::read_to_string(file_path) {
            for line in contents.lines() {
                let parts: Vec<&str> = line.split(": ").collect();
                if parts.len() == 2 {
                    if let (Ok(score), name) = (parts[1].parse::<usize>(), parts[0]) {
                        scores.push((name.to_string(), score));
                    }
                }
            }

            scores.sort_by(|a, b| b.1.cmp(&a.1)); // Ordena os scores do maior para o menor

            println!("Placar de líderes:");
            for (name, score) in scores {
                println!("{}: {}", name, score);
            }
        } else {
            println!("Sem pontuações ainda.");
        }
    }

    fn get_user_name() -> Option<String> {
        let mut name = String::new();
        println!("Digite o seu nome (ou pressione enter para pular):");
        io::stdin().read_line(&mut name).unwrap();
        let trimmed_name = name.trim().to_string();
        if trimmed_name.is_empty() {
            None
        } else {
            Some(trimmed_name)
        }
    }

    fn run(&mut self) {
        while !self.game_over {
            self.handle_input();
            self.update();
            self.update_buffer(); // Atualiza o buffer com o estado do jogo
            self.draw();
            sleep(Duration::from_millis(100));
        }
        println!("Fim de jogo! Final Score: {}", self.score);

        Self::display_scores();
        let name = Self::get_user_name();
        self.save_score(name.as_deref());
    }
}

// Função para exibir o menu principal
fn show_menu() -> usize {
    loop {
        let mut stdout = io::stdout();
        stdout.execute(cursor::MoveTo(0, 0)).unwrap();
        stdout.execute(Clear(ClearType::All)).unwrap();
        
        println!("Menu:");
        println!("1. Novo Jogo");
        println!("2. Highscores");
        println!("3. Sair");

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        match input.trim() {
            "1" => return 1,
            "2" => return 2,
            "3" => return 3,
            _ => println!("Escolha inválida. Tente novamente."),
        }
    }
}

fn main() {
    loop {
        let choice = show_menu();
        match choice {
            1 => {
                let width = 30;  // Aumentado para criar um campo maior
                let height = 20; // Aumentado para criar um campo maior
                let mut game = SnakeGame::new(width, height);
                game.run();
            }
            2 => {
                SnakeGame::display_scores();
                println!("Pressione Enter para voltar ao menu...");
                let mut _dummy = String::new();
                io::stdin().read_line(&mut _dummy).unwrap();
            }
            3 => break,
            _ => unreachable!(),
        }
    }
}
