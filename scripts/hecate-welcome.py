#!/usr/bin/env python3
"""
HecateOS Welcome Application
Interactive first-boot setup and configuration wizard
"""

import os
import sys
import json
import subprocess
import time
from pathlib import Path
import curses
from curses import wrapper
import threading

class HecateWelcome:
    def __init__(self, stdscr):
        self.stdscr = stdscr
        self.current_step = 0
        self.total_steps = 7
        self.hardware_profile = {}
        self.user_choices = {}
        
        # Colors
        curses.start_color()
        curses.init_pair(1, curses.COLOR_CYAN, curses.COLOR_BLACK)    # Title
        curses.init_pair(2, curses.COLOR_GREEN, curses.COLOR_BLACK)   # Success
        curses.init_pair(3, curses.COLOR_YELLOW, curses.COLOR_BLACK)  # Warning
        curses.init_pair(4, curses.COLOR_RED, curses.COLOR_BLACK)     # Error
        curses.init_pair(5, curses.COLOR_MAGENTA, curses.COLOR_BLACK) # Logo
        curses.init_pair(6, curses.COLOR_WHITE, curses.COLOR_BLACK)   # Normal
        
        # Hide cursor
        curses.curs_set(0)
        self.stdscr.nodelay(0)
        
    def draw_logo(self, y_offset=0):
        """Draw HecateOS ASCII logo"""
        logo = [
            "╦ ╦┌─┐┌─┐┌─┐┌┬┐┌─┐╔═╗╔═╗",
            "╠═╣├┤ │  ├─┤ │ ├┤ ║ ║╚═╗",
            "╩ ╩└─┘└─┘┴ ┴ ┴ └─┘╚═╝╚═╝",
            "",
            "Welcome to HecateOS 24.04 LTS"
        ]
        
        height, width = self.stdscr.getmaxyx()
        
        for i, line in enumerate(logo):
            x = (width - len(line)) // 2
            y = y_offset + i
            if i < 3:
                self.stdscr.attron(curses.color_pair(5))
                self.stdscr.addstr(y, x, line)
                self.stdscr.attroff(curses.color_pair(5))
            else:
                self.stdscr.attron(curses.color_pair(1))
                self.stdscr.addstr(y, x, line)
                self.stdscr.attroff(curses.color_pair(1))
    
    def draw_progress(self, y_offset=7):
        """Draw progress bar"""
        height, width = self.stdscr.getmaxyx()
        progress_width = 50
        x_start = (width - progress_width) // 2
        
        # Progress text
        progress_text = f"Step {self.current_step + 1} of {self.total_steps}"
        self.stdscr.addstr(y_offset, (width - len(progress_text)) // 2, progress_text)
        
        # Progress bar
        filled = int((self.current_step / self.total_steps) * progress_width)
        bar = "█" * filled + "░" * (progress_width - filled)
        
        self.stdscr.attron(curses.color_pair(2))
        self.stdscr.addstr(y_offset + 1, x_start, bar)
        self.stdscr.attroff(curses.color_pair(2))
    
    def show_menu(self, title, options, description=""):
        """Display menu and get user selection"""
        self.stdscr.clear()
        self.draw_logo()
        self.draw_progress()
        
        height, width = self.stdscr.getmaxyx()
        current_selection = 0
        
        while True:
            # Draw title
            self.stdscr.attron(curses.color_pair(1) | curses.A_BOLD)
            self.stdscr.addstr(10, (width - len(title)) // 2, title)
            self.stdscr.attroff(curses.color_pair(1) | curses.A_BOLD)
            
            # Draw description
            if description:
                desc_lines = description.split('\n')
                for i, line in enumerate(desc_lines):
                    self.stdscr.addstr(12 + i, (width - len(line)) // 2, line)
            
            # Draw options
            start_y = 15 if description else 12
            for i, option in enumerate(options):
                x = (width - len(option[0]) - 4) // 2
                
                if i == current_selection:
                    self.stdscr.attron(curses.color_pair(2) | curses.A_REVERSE)
                    self.stdscr.addstr(start_y + i * 2, x, f"  {option[0]}  ")
                    self.stdscr.attroff(curses.color_pair(2) | curses.A_REVERSE)
                    
                    # Show description
                    if len(option) > 1:
                        self.stdscr.addstr(start_y + i * 2 + 1, 
                                         (width - len(option[1])) // 2, 
                                         option[1])
                else:
                    self.stdscr.addstr(start_y + i * 2, x, f"  {option[0]}  ")
            
            # Instructions
            instructions = "↑/↓: Navigate  |  Enter: Select  |  q: Quit"
            self.stdscr.addstr(height - 2, (width - len(instructions)) // 2, instructions)
            
            self.stdscr.refresh()
            
            # Handle input
            key = self.stdscr.getch()
            if key == curses.KEY_UP and current_selection > 0:
                current_selection -= 1
            elif key == curses.KEY_DOWN and current_selection < len(options) - 1:
                current_selection += 1
            elif key == ord('\n'):
                return current_selection
            elif key == ord('q'):
                return -1
    
    def detect_hardware(self):
        """Run hardware detection"""
        self.stdscr.clear()
        self.draw_logo()
        
        height, width = self.stdscr.getmaxyx()
        
        self.stdscr.attron(curses.color_pair(1))
        self.stdscr.addstr(8, (width - 30) // 2, "Detecting Hardware...")
        self.stdscr.attroff(curses.color_pair(1))
        
        # Animation while detecting
        spinner = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏']
        
        for i in range(20):
            self.stdscr.addstr(10, width // 2, spinner[i % len(spinner)])
            self.stdscr.refresh()
            time.sleep(0.1)
        
        # Run actual hardware detection
        try:
            result = subprocess.run(['/usr/local/bin/hecate-hardware-detect'],
                                  capture_output=True, text=True)
            
            # Load hardware profile
            profile_path = Path('/etc/hecate/hardware-profile.json')
            if profile_path.exists():
                with open(profile_path, 'r') as f:
                    self.hardware_profile = json.load(f)
                
                # Display summary
                self.stdscr.clear()
                self.draw_logo()
                
                self.stdscr.attron(curses.color_pair(2))
                self.stdscr.addstr(8, (width - 30) // 2, "✓ Hardware Detected!")
                self.stdscr.attroff(curses.color_pair(2))
                
                # Show key hardware
                hw_info = [
                    f"CPU: {self.hardware_profile.get('cpu', {}).get('model', 'Unknown')}",
                    f"RAM: {self.hardware_profile.get('memory', {}).get('total_gb', 'Unknown')} GB",
                    f"GPU: {self.hardware_profile.get('gpu', {}).get('model', 'None')}",
                    f"Profile: {self.hardware_profile.get('system', {}).get('profile', 'standard').upper()}"
                ]
                
                for i, info in enumerate(hw_info):
                    self.stdscr.addstr(11 + i, (width - len(info)) // 2, info)
                
                self.stdscr.addstr(height - 2, (width - 20) // 2, "Press any key to continue")
                self.stdscr.refresh()
                self.stdscr.getch()
                
        except Exception as e:
            self.stdscr.attron(curses.color_pair(4))
            self.stdscr.addstr(10, (width - 40) // 2, f"Error detecting hardware: {e}")
            self.stdscr.attroff(curses.color_pair(4))
            self.stdscr.refresh()
            time.sleep(2)
    
    def step_welcome(self):
        """Welcome screen"""
        options = [
            ("Start Setup", "Configure HecateOS for your system"),
            ("Skip Setup", "Use default settings")
        ]
        
        choice = self.show_menu(
            "Welcome to HecateOS",
            options,
            "Let's configure your system for optimal performance"
        )
        
        if choice == 1:  # Skip setup
            return False
        return True
    
    def step_profile_selection(self):
        """Select system profile"""
        recommended = self.hardware_profile.get('system', {}).get('profile', 'standard')
        
        options = [
            (f"Ultimate (Recommended)" if recommended == "ultimate" else "Ultimate",
             "AI/ML workstation with maximum performance"),
            (f"Workstation (Recommended)" if recommended == "workstation" else "Workstation",
             "Professional workstation for development"),
            (f"Gaming (Recommended)" if recommended == "gaming" else "Gaming",
             "Optimized for gaming and streaming"),
            (f"Developer (Recommended)" if recommended == "developer" else "Developer",
             "Software development environment"),
            (f"Server (Recommended)" if recommended == "server" else "Server",
             "Headless server configuration"),
            ("Automatic", "Let HecateOS decide based on hardware")
        ]
        
        choice = self.show_menu(
            "Select System Profile",
            options,
            f"Detected hardware suggests: {recommended.upper()} profile"
        )
        
        if choice >= 0:
            profiles = ["ultimate", "workstation", "gaming", "developer", "server", "auto"]
            self.user_choices['profile'] = profiles[choice]
    
    def step_optimizations(self):
        """Select optimizations to apply"""
        options = [
            ("Maximum Performance", "All optimizations, no power saving"),
            ("Balanced", "Good performance with some power efficiency"),
            ("Power Saver", "Reduce power consumption"),
            ("Custom", "Choose individual optimizations")
        ]
        
        choice = self.show_menu(
            "Performance Settings",
            options,
            "How should we optimize your system?"
        )
        
        if choice >= 0:
            modes = ["performance", "balanced", "powersave", "custom"]
            self.user_choices['optimization_mode'] = modes[choice]
    
    def step_nvidia_settings(self):
        """Configure NVIDIA settings if GPU detected"""
        if self.hardware_profile.get('gpu', {}).get('vendor') != 'nvidia':
            return  # Skip if no NVIDIA GPU
        
        options = [
            ("Gaming Mode", "Optimize for low latency and high FPS"),
            ("Creator Mode", "Optimize for stability and quality"),
            ("Compute Mode", "Optimize for CUDA/AI workloads"),
            ("Automatic", "Let HecateOS configure automatically")
        ]
        
        choice = self.show_menu(
            "NVIDIA GPU Configuration",
            options,
            f"GPU: {self.hardware_profile.get('gpu', {}).get('model', 'Unknown')}"
        )
        
        if choice >= 0:
            modes = ["gaming", "creator", "compute", "auto"]
            self.user_choices['nvidia_mode'] = modes[choice]
    
    def step_developer_tools(self):
        """Select development tools to install"""
        options = [
            ("Full Stack", "All languages and frameworks"),
            ("Web Development", "Node.js, Python, databases"),
            ("System Programming", "C/C++, Rust, Go"),
            ("Data Science", "Python, R, Julia, Jupyter"),
            ("DevOps", "Docker, Kubernetes, Terraform"),
            ("Minimal", "Just the basics")
        ]
        
        choice = self.show_menu(
            "Development Tools",
            options,
            "Which development stack would you like?"
        )
        
        if choice >= 0:
            stacks = ["full", "web", "systems", "datascience", "devops", "minimal"]
            self.user_choices['dev_stack'] = stacks[choice]
    
    def step_privacy_telemetry(self):
        """Configure privacy and telemetry settings"""
        options = [
            ("Maximum Privacy", "Disable all telemetry and reporting"),
            ("Minimal Telemetry", "Only critical error reporting"),
            ("Help Improve HecateOS", "Send anonymous usage statistics")
        ]
        
        choice = self.show_menu(
            "Privacy Settings",
            options,
            "Control what information is shared"
        )
        
        if choice >= 0:
            levels = ["maximum", "minimal", "full"]
            self.user_choices['privacy_level'] = levels[choice]
    
    def apply_configuration(self):
        """Apply all user configurations"""
        self.stdscr.clear()
        self.draw_logo()
        
        height, width = self.stdscr.getmaxyx()
        
        self.stdscr.attron(curses.color_pair(1))
        self.stdscr.addstr(8, (width - 30) // 2, "Applying Configuration...")
        self.stdscr.attroff(curses.color_pair(1))
        
        tasks = [
            "Applying system profile...",
            "Configuring optimizations...",
            "Setting up GPU...",
            "Installing tools...",
            "Finalizing settings..."
        ]
        
        for i, task in enumerate(tasks):
            self.stdscr.addstr(10 + i, (width - len(task)) // 2, f"⠿ {task}")
            self.stdscr.refresh()
            time.sleep(0.5)
            self.stdscr.addstr(10 + i, (width - len(task)) // 2 - 2, "✓")
            self.stdscr.refresh()
        
        # Save configuration
        config_path = Path('/etc/hecate/user-config.json')
        config_path.parent.mkdir(parents=True, exist_ok=True)
        with open(config_path, 'w') as f:
            json.dump(self.user_choices, f, indent=2)
        
        # Run apply script
        subprocess.run(['/usr/local/bin/hecate-apply-optimizations'], 
                      capture_output=True)
        
        self.stdscr.attron(curses.color_pair(2))
        self.stdscr.addstr(17, (width - 30) // 2, "✓ Configuration Complete!")
        self.stdscr.attroff(curses.color_pair(2))
        
        self.stdscr.addstr(height - 2, (width - 20) // 2, "Press any key to continue")
        self.stdscr.refresh()
        self.stdscr.getch()
    
    def show_complete(self):
        """Show completion screen"""
        self.stdscr.clear()
        self.draw_logo()
        
        height, width = self.stdscr.getmaxyx()
        
        self.stdscr.attron(curses.color_pair(2) | curses.A_BOLD)
        self.stdscr.addstr(8, (width - 30) // 2, "HecateOS Setup Complete!")
        self.stdscr.attroff(curses.color_pair(2) | curses.A_BOLD)
        
        info = [
            "",
            "Your system is now optimized and ready to use.",
            "",
            "Quick Commands:",
            "  hecate-info        - System information",
            "  hecate-benchmark   - Run performance tests",
            "  hecate-update      - Update HecateOS",
            "",
            "Documentation: https://hecate-os.dev/docs",
            "Community: https://discord.gg/hecate-os",
            "",
            "Enjoy HecateOS!"
        ]
        
        for i, line in enumerate(info):
            if line.startswith("  "):
                self.stdscr.attron(curses.color_pair(1))
                self.stdscr.addstr(10 + i, (width - len(line)) // 2, line)
                self.stdscr.attroff(curses.color_pair(1))
            else:
                self.stdscr.addstr(10 + i, (width - len(line)) // 2, line)
        
        self.stdscr.addstr(height - 2, (width - 20) // 2, "Press any key to exit")
        self.stdscr.refresh()
        self.stdscr.getch()
    
    def run(self):
        """Main application flow"""
        # Step 0: Welcome
        if not self.step_welcome():
            return
        
        # Step 1: Hardware detection
        self.current_step = 1
        self.detect_hardware()
        
        # Step 2: Profile selection
        self.current_step = 2
        self.step_profile_selection()
        
        # Step 3: Optimizations
        self.current_step = 3
        self.step_optimizations()
        
        # Step 4: NVIDIA settings (if applicable)
        self.current_step = 4
        self.step_nvidia_settings()
        
        # Step 5: Developer tools
        self.current_step = 5
        self.step_developer_tools()
        
        # Step 6: Privacy settings
        self.current_step = 6
        self.step_privacy_telemetry()
        
        # Apply configuration
        self.apply_configuration()
        
        # Show completion
        self.show_complete()

def main(stdscr):
    """Main entry point"""
    app = HecateWelcome(stdscr)
    app.run()

if __name__ == "__main__":
    # Check if running as root
    if os.geteuid() != 0:
        print("This application must be run as root")
        sys.exit(1)
    
    # Check if first run
    if Path('/etc/hecate/.firstboot_done').exists():
        response = input("HecateOS is already configured. Run setup again? (y/N): ")
        if response.lower() != 'y':
            sys.exit(0)
    
    # Run the welcome app
    wrapper(main)
    
    # Mark as complete
    Path('/etc/hecate/.firstboot_done').touch()