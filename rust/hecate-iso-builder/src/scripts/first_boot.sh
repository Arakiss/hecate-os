#!/bin/bash
# HecateOS First Boot Script

echo "Welcome to HecateOS!"
echo "Running first boot configuration..."

# Run benchmarks
if command -v hecate-bench >/dev/null 2>&1; then
    hecate-bench sysinfo
fi

# Show dashboard URL
echo ""
echo "Monitor Dashboard available at: http://localhost:9313"
echo ""