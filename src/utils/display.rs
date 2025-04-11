use colored::Colorize;
use super::metadata::PluginMetadata;

/// Print a banner with plugin metadata
pub fn print_banner(plugin_metadata: &PluginMetadata) {
    let raw_banner = format!("
888     888     d8888 888      8888888b.         d8888 8888888b.        d8888 8888888b.  
888     888    d88888 888      888   Y88b       d88888 888  'Y88b      d88888 888   Y88b 
888     888   d88P888 888      888    888      d88P888 888    888     d88P888 888    888 
Y88b   d88P  d88P 888 888      888   d88P     d88P 888 888    888    d88P 888 888   d88P 
 Y88b d88P  d88P  888 888      8888888P'     d88P  888 888    888   d88P  888 8888888P'  
  Y88o88P  d88P   888 888      888 T88b     d88P   888 888    888  d88P   888 888 T88b   
   Y888P  d8888888888 888      888  T88b   d8888888888 888  .d88P d8888888888 888  T88b  
    Y8P  d88P     888 88888888 888   T88b d88P     888 8888888P' d88P     888 888   T88b 

    {}
                                                                                         
{} v{}                                                                                    
{}                                                                                         
    ", "by Mainasara Tsowa <tsowamainasara@gmail.com>".green().bold(), plugin_metadata.name.bold().white(), plugin_metadata.version.bold().yellow(), plugin_metadata.description.bold().blue());
    println!("{}", raw_banner.blue());
} 