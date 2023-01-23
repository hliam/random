//! Makes a box in your terminal that follows your mouse around. Fancy stuff.
//! This was mostly to try using crossterm for something.

use std::io::{self, stdout, Write};
use std::thread::sleep;
use std::time::Duration;

use crossterm::{cursor, event, terminal, Command, ExecutableCommand, QueueableCommand};

struct TermBox {
    centre: (u16, u16),
    horz_pad: u16,
    vert_pad: u16,
}

impl TermBox {
    #[must_use]
    fn new(centre: (u16, u16), horz_pad: u16, vert_pad: u16) -> Self {
        Self {
            centre,
            horz_pad,
            vert_pad,
        }
    }

    #[must_use]
    fn width(&self) -> u16 {
        (self.horz_pad * 2 + 1)
            .min(self.centre.0 + self.horz_pad)
            .min(
                terminal::size().expect("failed to get terminal size").0 - self.centre.0
                    + self.horz_pad.saturating_sub(1),
            )
    }

    #[must_use]
    fn height(&self) -> u16 {
        // We don't need to account for it trying to draw off the bottom of the terminal because the
        // terminal itself just figures that out for us. It's fine. Don't worry about it.
        //
        // Maybe sacrificing cross-platform support or something but who knows.
        (self.vert_pad * 2 + 1).min(self.centre.1 + self.vert_pad)
    }

    #[must_use]
    fn top_left(&self) -> (u16, u16) {
        (
            self.centre
                .0
                .saturating_sub(self.horz_pad)
                .saturating_sub(1),
            self.centre
                .1
                .saturating_sub(self.vert_pad)
                .saturating_sub(1),
        )
    }

    #[must_use]
    fn bottom_left(&self) -> (u16, u16) {
        (
            self.centre
                .0
                .saturating_sub(self.horz_pad)
                .saturating_sub(1),
            self.centre.1 + self.vert_pad + 1,
        )
    }

    #[must_use]
    fn top_right(&self) -> (u16, u16) {
        (
            self.centre.0 + self.horz_pad + 1,
            self.centre
                .1
                .saturating_sub(self.vert_pad)
                .saturating_sub(1),
        )
    }
}

impl std::fmt::Display for TermBox {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let vert = "│";
        let horz = "─";

        let top_left = self.top_left();
        let bottom_left = self.bottom_left();
        let top_right = self.top_right();
        let width = self.width() as usize;

        // Top bar
        cursor::MoveTo(top_left.0, top_left.1).write_ansi(f)?;
        f.write_str("┌")?;
        f.write_str(&horz.repeat(width))?;
        f.write_str("┐")?;

        // Inner rows
        for row_number in 1..=self.height() {
            cursor::MoveTo(top_left.0, top_left.1 + row_number).write_ansi(f)?;
            f.write_str(vert)?;
            cursor::MoveTo(top_right.0, top_right.1 + row_number).write_ansi(f)?;
            f.write_str(vert)?;
        }

        // Bottom bar
        cursor::MoveTo(bottom_left.0, bottom_left.1).write_ansi(f)?;
        f.write_str("└")?;
        f.write_str(&horz.repeat(width))?;
        f.write_str("┘")
    }
}

fn run(horz_pad: u16, vert_pad: u16) -> io::Result<()> {
    use event::{
        Event, KeyCode, KeyEvent, KeyEventKind::Press, KeyModifiers, MouseEvent, MouseEventKind,
    };

    let mut stdout = stdout();
    stdout
        .queue(cursor::Hide)?
        .execute(event::EnableMouseCapture)?
        .queue(event::EnableFocusChange)?
        .execute(terminal::EnterAlternateScreen)?;

    let mut term_box = TermBox::new((0, 0), horz_pad, vert_pad);

    loop {
        match event::read()? {
            Event::Mouse(e) => match e {
                MouseEvent {
                    kind: MouseEventKind::Moved,
                    column,
                    row,
                    ..
                } if (column, row) != term_box.centre => {
                    term_box.centre = (column, row);

                    stdout.queue(terminal::Clear(terminal::ClearType::All))?;
                    write!(stdout, "{}", term_box)?;
                    stdout.flush()?;
                }
                _ => (),
            },

            Event::Key(e) => match e {
                KeyEvent {
                    kind: Press,
                    code: KeyCode::Esc,
                    ..
                }
                | KeyEvent {
                    kind: Press,
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => {
                    // Cleanup of the terminal window and flags happens in `main`.
                    return Ok(());
                }
                _ => (),
            },

            Event::FocusLost => {
                stdout.execute(terminal::Clear(terminal::ClearType::All))?;
            }

            Event::FocusGained => {
                write!(stdout, "{}", term_box)?;
                stdout.flush()?;
            }

            _ => (),
        }

        sleep(Duration::from_millis(20));
    }
}

fn main() {
    let res = run(16, 6);

    let reset_res = (|| {
        stdout()
            .queue(cursor::Show)?
            .queue(event::DisableMouseCapture)?
            .queue(event::DisableFocusChange)?
            .queue(terminal::LeaveAlternateScreen)?
            .flush()
    })();

    if let Err(e) = &res {
        println!("Error: {}", e);
    }
    if let Err(e) = reset_res {
        println!(
            "An error {}occurred while trying to reset terminal settings: {}",
            if res.is_err() { "also " } else { "" },
            e
        )
    }
}
