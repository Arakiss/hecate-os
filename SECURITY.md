# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Security Considerations

HecateOS is designed for **maximum performance**, which includes some security trade-offs:

### Mitigations Disabled
By default, HecateOS disables Spectre/Meltdown mitigations (`mitigations=off`) for 5-15% performance gain. This is intentional for workstation use but may not be appropriate for:
- Public-facing servers
- Multi-tenant environments
- Systems handling sensitive data

To re-enable mitigations:
```bash
# Edit GRUB config
sudo nano /etc/default/grub
# Remove "mitigations=off" from GRUB_CMDLINE_LINUX_DEFAULT
sudo update-grub
sudo reboot
```

### Other Security Notes
- SSH server is enabled by default
- Firewall (ufw) is installed but not enabled by default
- Docker runs with NVIDIA GPU access

## Reporting a Vulnerability

If you discover a security vulnerability in HecateOS:

1. **Do NOT open a public issue**
2. Email the maintainer directly (see GitHub profile)
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

You can expect:
- Acknowledgment within 48 hours
- Status update within 7 days
- Credit in the fix (unless you prefer anonymity)

## Security Updates

Security updates for underlying Ubuntu packages are applied via:
```bash
sudo apt update && sudo apt upgrade
```

HecateOS inherits Ubuntu 24.04 LTS security support until April 2029.
