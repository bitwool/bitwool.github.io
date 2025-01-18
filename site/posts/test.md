+++
title: 测试博客
date: 2025-01-08T17:15:12+08:00
tags: tag1,tag2
+++

## golang

依赖库：https://github.com/urfave/cli

项目初始化过程

```bash
mkdir cli-app
cd cli-app 
go mod init cli-app
go get github.com/urfave/cli/v3
touch main.go
go build .
./cli-app
```

main.go
``` go
package main

import (
	"context"
	"fmt"
	"log"
	"os"

	"github.com/urfave/cli/v3"
)

func main() {
	cmd := &cli.Command{
		Name:  "boom",
		Usage: "make an explosive entrance",
		Action: func(context.Context, *cli.Command) error {
			fmt.Println("boom! I say!")
			return nil
		},
	}

	if err := cmd.Run(context.Background(), os.Args); err != nil {
		log.Fatal(err)
	}
}	
```

## ocaml

- 依赖库：https://erratique.ch/software/cmdliner
- 使用方法 https://github.com/mjambon/cmdliner-cheatsheet