package main

import (
	"fmt"
	"os"
	"time"

	"github.com/tjan147/wrapper"
)

func main() {
	// sentinel call
	wrapper.CallSentinel("tjan@golang.TestCallRustSample()")

	err := os.Mkdir("../sample", 0755)
	if err != nil {
		panic(err)
	}

	start := time.Now()
	ret := wrapper.CallSetup("../rust/sample/sample.txt", "../sample")
	fmt.Printf("CallSetup() = %d, takes %s\n", ret, time.Now().Sub(start).String())
}
