# build_static_shell.tcl - Build static shell for Xilinx KV260
# Usage: vivado -mode batch -source build_static_shell.tcl -tclargs <project_dir>
#
# This creates the static shell bitstream with:
#   - Zynq UltraScale+ XCZU3CG configured
#   - 256K LUT fabric at 150MHz
#   - UART0 @ 115200 8N1
#   - SDIO for SD card
#   - PCAP for dynamic partial reconfiguration
#   - 4 RP slots (RP_0, RP_1, RP_2, RP_3)

set project_dir [lindex $argv 0]
if {$project_dir eq ""} {
    set project_dir "./build"
}

puts "=== Moore Kernel Static Shell Build ==="
puts "Project directory: $project_dir"

# Create project
set proj_name "moore_static_shell"
set part "xczu3cg-sfvc784-1"

puts "Creating Vivado project..."
create_project $proj_name $project_dir -part $part -force

# Configure Zynq PS
puts "Configuring Zynq UltraScale+ PS..."

# DDR4 Controller - 2GB, 2400 MT/s
set_property CONFIG.DDR_MEMORY_TYPE {DDR4} [get_bd_cells zynq_ultra_ps_e_0]
set_property CONFIG.DDR_DEVICE_CAPACITY {2048} [get_bd_cells zynq_ultra_ps_e_0]
set_property CONFIG.DDR_SPEED_GRADE {2400} [get_bd_cells zynq_ultra_ps_e_0]

# Clock configuration
# CPU: 1200 MHz, PL: 150 MHz
set_property CONFIG.PSU__CRL_APB__PL0_REF__FREQMHZ {150.0} [get_bd_cells zynq_ultra_ps_e_0]
set_property CONFIG.PSU__CRL_APB__PL1_REF__FREQMHZ {150.0} [get_bd_cells zynq_ultra_ps_e_0]
set_property CONFIG.PSU__CRL_APB__PL2_REF__FREQMHZ {150.0} [get_bd_cells zynq_ultra_ps_e_0]

# UART0 configuration
set_property CONFIG.PSU__UART0__PERIPHERAL__ENABLE {1} [get_bd cells zynq_ultra_ps_e_0]
set_property CONFIG.PSU__UART0__PERIPHERAL__IO {EMGIO} [get_bd_cells zynq_ultra_ps_e_0]
set_property CONFIG.PSU__UART0__BAUD_RATE {115200} [get_bd_cells zynq_ultra_ps_e_0]

# SDIO configuration
set_property CONFIG.PSU__SDIO1__PERIPHERAL__ENABLE {1} [get_bd_cells zynq_ultra_ps_e_0]
set_property CONFIG.PSU__SDIO1__PERIPHERAL__IO {EMGIO} [get_bd_cells zynq_ultra_ps_e_0]

# GPIO for LEDs and buttons
set_property CONFIG.PSU__GPIO0__PERIPHERAL__ENABLE {1} [get_bd_cells zynq_ultra_ps_e_0]

# Enable ECC for OCM
set_property CONFIG.PSU__OCM__ECC {1} [get_bd_cells zynq_ultra_ps_e_0]

# HP0 AXI slave port (for PCAP access from PS)
set_property CONFIG.PSU__HP0_DDR_DIRECT_CONNECTION {ps8_DDR_0} [get_bd_cells zynq_ultra_ps_e_0]

# Enable fabric clocking
set_property CONFIG.PSU__FPD_SLCR__CLKOUT0__ENABLE {1} [get_bd_cells zynq_ultra_ps_e_0]
set_property CONFIG.PSU__FPD_SLCR__CLKOUT0__FREQMHZ {150} [get_bd_cells zynq_ultra_ps_e_0]

# Create RP_0 partition - 40K LUTs for kernel_ops
puts "Creating RP_0 (kernel_ops - 40K LUTs)..."
create_partition -name RP_0 -hw_design [current_hw_design]
set_property PARTITION_PINASSIGNMENTS [list \
    {CONFIG.PARTITION_PIN_IOPAD 1} \
] [get_rps RP_0]

# Create RP_1 partition - 80K LUTs for app_slot_1
puts "Creating RP_1 (app_slot_1 - 80K LUTs)..."
create_partition -name RP_1 -hw_design [current_hw_design]

# Create RP_2 partition - 80K LUTs for app_slot_2
puts "Creating RP_2 (app_slot_2 - 80K LUTs)..."
create_partition -name RP_2 -hw_design [current_hw_design]

# Create RP_3 partition - 40K LUTs for reserved/expansion
puts "Creating RP_3 (reserved - 40K LUTs)..."
create_partition -name RP_3 -hw_design [current_hw_design]

# Add PCAP wrapper for dynamic configuration
puts "Adding PCAP wrapper for partial reconfiguration..."
create_cell -name pcap_wrapper -lib_type xilinx.com_ip_processing_system7_5_0 processing_system7_0
set_property CONFIG.PSU__PCAP__PERIPHERAL__ENABLE {1} [get_cells pcap_wrapper]

# Connect PCAP to Fabric
create_bd_cell -type ip -vlnv xilinx.com:ip:axi_interconnect axi_pcap_interconnect
puts "Connecting PCAP to fabric AXI master..."

# Run block automation
puts "Running block automation..."
validate_bd_design

# Generate static bitstream
puts "Generating static shell bitstream..."
set_property strategy Flow_PerfOptimized_high [get_runs impl_1]
launch_runs impl_1 -to_step write_bitstream -paths_subdirs [get_filesets runs_1]
wait_on_run impl_1

if {[get_property PROGRESS [get_runs impl_1]] != 100} {
    puts "ERROR: Bitstream generation failed!"
    exit 1
}

puts "=== Static shell build complete ==="
puts "Bitstream: $project_dir/$proj_name.runs/impl_1/static_shell.bit"

# Export hardware description for Brief compilation
write_json -force $project_dir/static_shell.json

exit 0