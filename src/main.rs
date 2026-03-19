use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
    Terminal,
};
use std::io;
use sysinfo::{Disks, System, Networks};

fn main() -> Result<(), io::Error> {
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // New sysinfo 0.30 API: Disks and Networks are handled differently
    let disks = Disks::new_with_refreshed_list();
    let networks = Networks::new_with_refreshed_list();
    
    // Storage logic
    let disk = disks.iter().find(|d| d.mount_point().to_str() == Some("/data"))
        .or_else(|| disks.iter().next());
        
    let (perc, label) = disk.map_or((0, "N/A".to_string()), |d| {
        let total = d.total_space();
        let used = total - d.available_space();
        let p = (used as f64 / total as f64 * 100.0) as u16;
        (p, format!("{:.0}G/{:.0}G", used as f64 / 1e9, total as f64 / 1e9))
    });

    // WireGuard detection using new Networks API
    let mut wg_status = "down".to_string();
    for (interface_name, _) in &networks {
        if interface_name.contains("tun") || interface_name.contains("wg") {
            wg_status = "10.2.0.2 (UP)".to_string();
        }
    }

    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), 
                Constraint::Length(3), 
                Constraint::Min(3),    
            ])
            .split(f.size());

        let title_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);

        // Box 1: Storage
        f.render_widget(Gauge::default()
            .block(Block::default().title(" [STORAGE STATUS] ").title_style(title_style).borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)))
            .gauge_style(Style::default().fg(Color::Magenta))
            .percent(perc)
            .label(label), chunks[0]);

        // Box 2: Network
        f.render_widget(Paragraph::new(format!(" WG: {}", wg_status))
            .block(Block::default().title(" [NETWORK STATUS] ").title_style(title_style).borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan))), chunks[1]);

        // Box 3: Home (Fixed the "ls" quoting error)
        f.render_widget(Paragraph::new(" Run ls to view files")
            .block(Block::default().title(" [$HOME] ").title_style(title_style).borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan))), chunks[2]);
    })?;

    std::thread::sleep(std::time::Duration::from_millis(1500));
    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}