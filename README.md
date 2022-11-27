# Inverse Index
This is a repository for a parallel computing university course project.

## Running
### Prerequisites
In order to build and run this project, you will need to have Rust install. It is recommended that you follow the [official instructions](https://www.rust-lang.org/tools/install) on how to do so.

In order to clone the repository, you will need to have [Git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git) installed.

### Building
To build the project, run the following commands:

```
# check out the repository
git clone https://github.com/BardiTheWeird/parallel-computing-coursework
cd parallel-computing-coursework

# build the project
cargo build --release --bins
```

With these run, you should have both server and cli client built. You can find their respective binaries at `./target/release`

### Running
#### Server
```
Usage: parallel_computing.exe <COMMAND>

Commands:
  time
  serve
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help information
```

##### Serving
```
Usage: parallel_computing.exe serve [OPTIONS]

Options:
  -s, --server-address <SERVER_ADDRESS>  [default: 127.0.0.1:8080]
  -d, --directory <DIRECTORIES>
  -t, --thread-count <THREAD_COUNT>      [default: 1]
  -h, --help                             Print help information
```

##### Timing
Server binary also supports timing the creation of the inverse index using text files in the specified directories

```
Usage: parallel_computing.exe time [OPTIONS] --thread-start <THREAD_COUNT_START> --thread-end <THREAD_COUNT_END>

Options:
  -d, --directory <DIRECTORIES>
      --thread-start <THREAD_COUNT_START>
      --thread-end <THREAD_COUNT_END>
  -o <OUTPUT_FORMAT>                       [default: json] [possible values: json, yaml]
  -i <ITERATIONS>                          [default: 10]
  -h, --help                               Print help information
```

#### Client
```
Usage: cli_client.exe [OPTIONS]

Options:
  -s, --server-address <SERVER_ADDRESS>  [default: 127.0.0.1:8080]
  -r, --request-kind <REQUEST_KIND>      [default: ping] [possible values: ping, index, file]
  -p, --payload <PAYLOAD>
  -h, --help                             Print help information
```