// Test system functions FFI implementation

function test_environment_variables() {
  println("=== Testing Environment Variables ===");
  
  // Test set environment variable
  let set_result = set_env("TEST_VAR", "test_value");
  println("Set TEST_VAR: " + to_string(set_result));
  
  // Test get environment variable
  let get_result = get_env("TEST_VAR");
  println("Get TEST_VAR: " + to_string(get_result));
  
  // Test delete environment variable
  let unset_result = unset_env("TEST_VAR");
  println("Unset TEST_VAR: " + to_string(unset_result));
  
  // Test get deleted environment variable
  let get_result2 = get_env("TEST_VAR");
  println("Get deleted TEST_VAR: " + to_string(get_result2));
  
  // Test get all environment variables
  let all_vars = env_vars();
  println("Environment variables count: " + to_string(len(all_vars)));
  if len(all_vars) > 0 {
    println("First environment variable: " + to_string(all_vars[0]));
  }
  
  println();
}

function test_command_line_args() {
  println("=== Testing Command Line Args ===");
  
  let args_list = args();
  println("Command line args count: " + to_string(len(args_list)));
  println("Command line args: " + to_string(args_list));
  
  let count = arg_count();
  println("arg_count(): " + to_string(count));
  
  for i in 0..count {
    let arg_val = arg(i);
    println("arg(" + to_string(i) + "): " + to_string(arg_val));
  }
  
  println();
}

function test_process_operations() {
  println("=== Testing Process Operations ===");
  
  let pid = getpid();
  println("Current process ID: " + to_string(pid));
  
  let ppid = getppid();
  println("Parent process ID: " + to_string(ppid));
  
  // Test execute system command
  let system_result = system("echo Hello from system command");
  println("system() return value: " + to_string(system_result));
  
  // Test execute command and get output
  let output_result = command_output("echo Hello from command_output");
  println("command_output() return value: " + to_string(output_result));
  
  println();
}

function test_system_info() {
  println("=== Testing System Info ===");
  
  let os = os_type();
  println("OS type: " + os);
  
  let version = os_version();
  println("OS version: " + version);
  
  let host = hostname();
  println("Hostname: " + host);
  
  let architecture = arch();
  println("Architecture: " + architecture);
  
  let free_mem = free_memory();
  println("Free memory: " + to_string(free_mem) + " bytes");
  
  let total_mem = total_memory();
  println("Total memory: " + to_string(total_mem) + " bytes");
  
  let cpus = cpu_count();
  println("CPU count: " + to_string(cpus));
  
  println();
}

function test_path_operations() {
  println("=== Testing Path Operations ===");
  
  let current = current_dir();
  println("Current directory: " + current);
  
  // Test path join
  let paths = ["dir1", "dir2", "file.txt"];
  let joined = path_join(paths);
  println("Path join result: " + joined);
  
  // Test path decomposition
  let test_path = current + "/test/path/file.txt";
  let dirname = path_dirname(test_path);
  println("Path dirname: " + dirname);
  
  let basename = path_basename(test_path);
  println("Path basename: " + basename);
  
  let extension = path_extension(test_path);
  println("Path extension: " + extension);
  
  // Test path existence
  let exists = path_exists(current);
  println("Current directory exists: " + to_string(exists));
  
  let is_file_result = is_file(current);
  println("Current directory is file: " + to_string(is_file_result));
  
  let is_dir_result = is_dir(current);
  println("Current directory is dir: " + to_string(is_dir_result));
  
  println();
}

function test_temp_files() {
  println("=== Testing Temp Files ===");
  
  let temp_file_result = temp_file();
  println("Create temp file: " + to_string(temp_file_result));
  
  let temp_dir_result = temp_dir();
  println("Create temp dir: " + to_string(temp_dir_result));
  
  let temp_dir_path = get_temp_dir();
  println("System temp dir: " + temp_dir_path);
  
  println();
}

function test_random_numbers() {
  println("=== Testing Random Numbers ===");
  
  // Set random seed
  srand(12345);
  println("Set random seed to 12345");
  
  // Generate random integers
  for i in 0..5 {
    let rand_num = random(100);
    println("Random integer (0-99): " + to_string(rand_num));
  }
  
  // Generate random floats
  for i in 0..5 {
    let rand_float = random_float();
    println("Random float (0.0-1.0): " + to_string(rand_float));
  }
  
  println();
}

function test_other_system_functions() {
  println("=== Testing Other System Functions ===");
  
  let uptime_val = uptime();
  println("System uptime: " + to_string(uptime_val) + " seconds");
  
  let uid = getuid();
  println("User ID: " + to_string(uid));
  
  let gid = getgid();
  println("Group ID: " + to_string(gid));
  
  let username = get_username();
  println("Username: " + username);
  
  let groupname = get_groupname();
  println("Groupname: " + groupname);
  
  println();
}

function main() {
  println("Starting FFI implementation test...");
  println("====================================");
  
  test_environment_variables();
  test_command_line_args();
  test_process_operations();
  test_system_info();
  test_path_operations();
  test_temp_files();
  test_random_numbers();
  test_other_system_functions();
  
  println("====================================");
  println("System functions test completed!");
}

main();
