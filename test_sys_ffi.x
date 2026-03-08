// 测试系统函数的FFI实现

function test_environment_variables() {
  println("=== 测试环境变量 ===");
  
  // 测试设置环境变量
  let set_result = set_env("TEST_VAR", "test_value");
  println("设置环境变量 TEST_VAR: " + to_string(set_result));
  
  // 测试获取环境变量
  let get_result = get_env("TEST_VAR");
  println("获取环境变量 TEST_VAR: " + to_string(get_result));
  
  // 测试删除环境变量
  let unset_result = unset_env("TEST_VAR");
  println("删除环境变量 TEST_VAR: " + to_string(unset_result));
  
  // 测试获取已删除的环境变量
  let get_result2 = get_env("TEST_VAR");
  println("获取已删除的环境变量 TEST_VAR: " + to_string(get_result2));
  
  // 测试获取所有环境变量
  let all_vars = env_vars();
  println("环境变量数量: " + to_string(len(all_vars)));
  if len(all_vars) > 0 {
    println("第一个环境变量: " + to_string(all_vars[0]));
  }
  
  println();
}

function test_command_line_args() {
  println("=== 测试命令行参数 ===");
  
  let args_list = args();
  println("命令行参数数量: " + to_string(len(args_list)));
  println("命令行参数: " + to_string(args_list));
  
  let count = arg_count();
  println("arg_count(): " + to_string(count));
  
  for i in 0..count {
    let arg_val = arg(i);
    println("arg(" + to_string(i) + "): " + to_string(arg_val));
  }
  
  println();
}

function test_process_operations() {
  println("=== 测试进程操作 ===");
  
  let pid = getpid();
  println("当前进程ID: " + to_string(pid));
  
  let ppid = getppid();
  println("父进程ID: " + to_string(ppid));
  
  // 测试执行系统命令
  let system_result = system("echo Hello from system command");
  println("system() 返回值: " + to_string(system_result));
  
  // 测试执行命令并获取输出
  let output_result = command_output("echo Hello from command_output");
  println("command_output() 返回值: " + to_string(output_result));
  
  println();
}

function test_system_info() {
  println("=== 测试系统信息 ===");
  
  let os = os_type();
  println("操作系统类型: " + os);
  
  let version = os_version();
  println("操作系统版本: " + version);
  
  let host = hostname();
  println("主机名: " + host);
  
  let architecture = arch();
  println("系统架构: " + architecture);
  
  let free_mem = free_memory();
  println("可用内存: " + to_string(free_mem) + " 字节");
  
  let total_mem = total_memory();
  println("总内存: " + to_string(total_mem) + " 字节");
  
  let cpus = cpu_count();
  println("CPU核心数: " + to_string(cpus));
  
  println();
}

function test_path_operations() {
  println("=== 测试路径操作 ===");
  
  let current = current_dir();
  println("当前工作目录: " + current);
  
  // 测试路径拼接
  let paths = ["dir1", "dir2", "file.txt"];
  let joined = path_join(paths);
  println("路径拼接结果: " + joined);
  
  // 测试路径分解
  let test_path = current + "/test/path/file.txt";
  let dirname = path_dirname(test_path);
  println("路径目录部分: " + dirname);
  
  let basename = path_basename(test_path);
  println("路径文件名部分: " + basename);
  
  let extension = path_extension(test_path);
  println("路径扩展名: " + extension);
  
  // 测试路径存在性
  let exists = path_exists(current);
  println("当前目录存在: " + to_string(exists));
  
  let is_file_result = is_file(current);
  println("当前目录是文件: " + to_string(is_file_result));
  
  let is_dir_result = is_dir(current);
  println("当前目录是目录: " + to_string(is_dir_result));
  
  println();
}

function test_temp_files() {
  println("=== 测试临时文件 ===");
  
  let temp_file_result = temp_file();
  println("创建临时文件: " + to_string(temp_file_result));
  
  let temp_dir_result = temp_dir();
  println("创建临时目录: " + to_string(temp_dir_result));
  
  let temp_dir_path = get_temp_dir();
  println("系统临时目录: " + temp_dir_path);
  
  println();
}

function test_random_numbers() {
  println("=== 测试随机数 ===");
  
  // 设置随机数种子
  srand(12345);
  println("设置随机数种子为 12345");
  
  // 生成随机整数
  for i in 0..5 {
    let rand_num = random(100);
    println("随机整数 (0-99): " + to_string(rand_num));
  }
  
  // 生成随机浮点数
  for i in 0..5 {
    let rand_float = random_float();
    println("随机浮点数 (0.0-1.0): " + to_string(rand_float));
  }
  
  println();
}

function test_other_system_functions() {
  println("=== 测试其他系统函数 ===");
  
  let uptime_val = uptime();
  println("系统启动时间: " + to_string(uptime_val) + " 秒");
  
  let uid = getuid();
  println("用户ID: " + to_string(uid));
  
  let gid = getgid();
  println("组ID: " + to_string(gid));
  
  let username = get_username();
  println("用户名: " + username);
  
  let groupname = get_groupname();
  println("组名: " + groupname);
  
  println();
}

function main() {
  println("开始测试系统函数的FFI实现...");
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
  println("系统函数测试完成！");
}

main();
