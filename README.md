# archiverd

`archiverd` is a lightweight daemon designed to monitor and archive files in a specified directory. It's particularly useful in scenarios where files are periodically created or updated, and there's a need to archive older versions automatically. This tool is ideal for managing log files, packet captures, and similar data.

## Usecase

Consider a folder where new files are regularly added or updated. For instance:

```
$ ls test/
testfile1.txt
```

With `archiverd`, when `testfile2.txt` is created, the older file (`testfile1.txt`) is automatically archived. . `archiverd` can be especially beneficial for maintaining orderly log files or packet captures.

This is the folder structure after `testfile2.txt` file is created:

```
$ ls test/
testfile1.txt.tar.gz
testfile2.ttt
```

## Usage

```
Usage: archiverd [OPTIONS] --directory <DIRECTORY>

Options:
  -d, --directory <DIRECTORY>   Specify the directory to monitor.
  -m, --max-files <MAX_FILES>   Set the maximum number of files to keep unarchived.
  -e, --exclude <EXCLUDE>       Define patterns to exclude files from archiving.
  -h, --help                    Display this help message.
  -V, --version                 Show the version of the archiverd.
```
