// X语言标准库 - 主入口
//
// 标准库的统一入口点，导入并重新导出所有模块

// ==========================================
// 导入各个模块
// ==========================================

import "./prelude"
import "./option"
import "./result"
import "./math"
import "./string"
import "./collections"
import "./iter"
import "./io"
import "./time"
import "./sys"

// ==========================================
// 重新导出 prelude 模块
// ==========================================

export assert

export assert_eq

export assert_ne

export assert_true

export assert_false

export assert_none

export assert_some

export assert_ok

export assert_err

export to_string

export to_int

export to_float

export to_bool

export print

export println

export eprint

export eprintln

export panic

export todo

export unimplemented

export dbg

// ==========================================
// 重新导出 option 模块
// ==========================================

export type Option

export Some

export None

export is_some

export is_none

export unwrap

export unwrap_or

export unwrap_or_else

export map

export and_then

export or

export or_else

export filter

export zip

export ok_or

export ok_or_else

export flatten

export as_ref

export as_mut

export contains

export replace

export take

// ==========================================
// 重新导出 result 模块
// ==========================================

export type Result

export Ok

export Err

export is_ok

export is_err

export unwrap

export unwrap_err

export unwrap_or

export unwrap_or_else

export map

export map_err

export and_then

export or_else

export flatten

export as_ref

export as_mut

export contains

export replace

export take

export expect

export expect_err

// ==========================================
// 重新导出 math 模块
// ==========================================

export PI

export E

export INF

export NAN

export MIN_INT

export MAX_INT

export MIN_FLOAT

export MAX_FLOAT

export abs

export max

export min

export clamp

export signum

export sqrt

export cbrt

export hypot

export sin

export cos

export tan

export asin

export acos

export atan

export atan2

export sinh

export cosh

export tanh

export exp

export exp2

export expm1

export ln

export log2

export log10

export log1p

export pow

export floor

export ceil

export round

export trunc

export fract

export modf

export fmod

export remainder

export degrees

export radians

export lerp

export smoothstep

export step

export factorial

export gcd

export lcm

export is_prime

export next_prime

export fibonacci

export binomial

export permutation

export combination

export random_int

export random_float

export random_range

// ==========================================
// 重新导出 string 模块
// ==========================================

export len

export is_empty

export char_at

export set_char

export substring

export starts_with

export ends_with

export contains

export find

export rfind

export replace

export replace_all

export split

export split_whitespace

export join

export trim

export trim_start

export trim_end

export to_upper

export to_lower

export capitalize

export reverse

export is_alpha

export is_digit

export is_alphanumeric

export is_whitespace

export is_ascii

export parse_int

export parse_float

export parse_bool

export format

export format_int

export format_float

export repeat

export pad_start

export pad_end

export center

export escape

export unescape

export bytes

export from_bytes

export chars

export from_chars

export lines

export words

export is_palindrome

export levenshtein_distance

export hamming_distance

export common_prefix

export common_suffix

// ==========================================
// 重新导出 collections 模块
// ==========================================

export type List

export type Map

export type Set

export list

export map

export set

export len

export is_empty

export push

export pop

export insert

export remove

export get

export get_mut

export contains

export clear

export append

export extend

export reverse

export sort

export sort_by

export filter

export map_list

export fold

export find

export find_index

export any

export all

export zip

export unzip

export flatten

export unique

export dedup

export rotate

export split_at

export join

export keys

export values

export entries

export update

export merge

export from_list

export to_list

export from_array

export to_array

export iter

export iter_mut

export into_iter

// ==========================================
// 重新导出 iter 模块
// ==========================================

export type Iterator

export iter

export next

export map

export filter

export fold

export find

export find_map

export any

export all

export none

export count

export sum

export product

export max

export min

export collect

export collect_list

export collect_map

export collect_set

export zip

export chain

export cycle

export take

export take_while

export skip

export skip_while

export enumerate

export rev

export sorted

export distinct

export flatten

export flat_map

export intersperse

export for_each

export inspect

export try_fold

export try_for_each

export position

export rposition

export nth

export last

export partition

export unzip

export scan

export fuse

export peekable

export multipeek

export cloned

export copied

export by_ref

export to_string

export from_list

export from_range

export range

export repeat

export once

export empty

// ==========================================
// 重新导出 io 模块
// ==========================================

export print

export println

export eprint

export eprintln

export print_char

export print_str

export read_char

export read_line

export read_input

export read_file

export write_file

export append_file

export read_lines

export write_lines

export copy_file

export move_file

export delete_file

export exists

export is_file

export is_dir

export create_dir

export create_dir_all

export remove_dir

export remove_dir_all

export list_dir

export current_dir

export change_dir

export canonicalize

export temp_file

export temp_dir

export stdin

export stdout

export stderr

export File

export open

export create

export append

export read

export write

export seek

export tell

export flush

export close

export buffer

export with_buffer

export memory

export with_memory

// ==========================================
// 重新导出 time 模块
// ==========================================

export type Time

export type Duration

export type DateTime

export NANOS_PER_SECOND

export NANOS_PER_MILLISECOND

export NANOS_PER_MICROSECOND

export SECONDS_PER_MINUTE

export SECONDS_PER_HOUR

export SECONDS_PER_DAY

export timestamp

export timestamp_millis

export timestamp_micros

export timestamp_nanos

export now

export sleep

export sleep_ms

export sleep_us

export sleep_ns

export sleep_duration

export duration_seconds

export duration_millis

export duration_micros

export duration_nanos

export duration_minutes

export duration_hours

export duration_days

export duration_as_seconds

export duration_as_millis

export duration_as_micros

export duration_as_nanos

export duration_add

export duration_sub

export duration_compare

export time_diff

export time_add

export time_sub

export time_compare

export to_local_datetime

export to_utc_datetime

export from_datetime

export local_now

export utc_now

export format_datetime

export format_iso8601

export datetime_to_string

export weekday_name

export weekday_abbr

export month_name

export month_abbr

export time_it

export time_it_print

// ==========================================
// 重新导出 sys 模块
// ==========================================

export get_env

export set_env

export unset_env

export env_vars

export args

export arg_count

export arg

export getpid

export getppid

export exit

export system

export command_output

export os_type

export os_version

export hostname

export arch

export free_memory

export total_memory

export cpu_count

export current_dir

export chdir

export path_join

export path_dirname

export path_basename

export path_extension

export path_exists

export is_file

export is_dir

export temp_file

export temp_dir

export get_temp_dir

export type Signal

export SIGINT

export SIGTERM

export SIGKILL

export SIGSEGV

export signal

export kill

export syscall

export uptime

export random

export random_float

export srand

export getuid

export getgid

export get_username

export get_groupname

// ==========================================
// 版本信息
// ==========================================

/// 标准库版本
export let VERSION: String = "1.0.0"

/// 标准库名称
export let NAME: String = "X Standard Library"

/// 标准库描述
export let DESCRIPTION: String = "X语言标准库，提供核心功能和工具"
