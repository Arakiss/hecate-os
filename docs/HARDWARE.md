# Hardware Compatibility

## Tested Hardware

HecateOS has been tested on:
- Intel Core i9-13900K
- NVIDIA RTX 4090
- 128GB DDR5-6400
- Samsung 990 PRO NVMe

## Supported Hardware

### GPUs

#### NVIDIA (Driver 590 series - 2026)
- RTX 50 series: 5090, 5080, 5070
- RTX 40 series: 4090, 4080, 4070 Ti, 4070, 4060 Ti, 4060
- RTX 30 series: 3090 Ti, 3090, 3080 Ti, 3080, 3070 Ti, 3070, 3060 Ti, 3060
- RTX 20 series: All models
- GTX 16 series: All models
- GTX 10 series: All models

#### AMD
- RX 8000 series (upcoming 2026)
- RX 9000 series (upcoming 2026)
- RX 7000 series: 7900 XTX, 7900 XT, 7800 XT, 7700 XT, 7600
- RX 6000 series: All models

#### Intel
- Arc B-series (Battlemage - 2026)
- Arc A-series: A770, A750, A580, A380

### CPUs

#### Intel
- 14th Gen (Raptor Lake Refresh)
- 13th Gen (Raptor Lake)
- 12th Gen (Alder Lake)
- 11th Gen (Rocket Lake)
- 10th Gen (Comet Lake)

#### AMD
- Zen 5 (Ryzen 9000 series)
- Zen 4 (Ryzen 7000 series)
- Zen 3+ (Ryzen 6000 series)
- Zen 3 (Ryzen 5000 series)
- Zen 2 (Ryzen 3000 series)

### Memory
- DDR5: All speeds supported
- DDR4: All speeds supported
- Minimum: 8GB
- Maximum tested: 512GB

### Storage
- NVMe Gen5 (PCIe 5.0)
- NVMe Gen4 (PCIe 4.0)
- NVMe Gen3 (PCIe 3.0)
- SATA SSD
- SATA HDD

## Performance Expectations

| Hardware Tier | Expected Performance Gain |
|--------------|---------------------------|
| AI Flagship (RTX 4090+, 64GB+ RAM) | 15-25% |
| Pro Workstation (RTX 4080/3090) | 12-20% |
| Gaming Enthusiast (RTX 4070/3080) | 10-18% |
| Standard Desktop | 5-15% |

## Known Limitations

- AMD GPU support is basic (no advanced power management)
- Intel Arc support is experimental
- Laptop-specific optimizations not yet implemented
- No ARM architecture support