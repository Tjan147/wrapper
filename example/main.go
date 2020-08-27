package main

import (
	"fmt"
	"os"
	"path"

	"github.com/tjan147/wrapper"
)

const (
	exampleAttr = 0755
	exampleSize = 1024
)

var (
	exampleDir  = "../sample"
	exampleName = "sample.dat"
)

func main() {
	exampleFile := path.Join(exampleDir, exampleName)

	// TODO: replace this with wrapper api later
	if err := os.Mkdir(exampleDir, exampleAttr); err != nil {
		panic(err)
	}

	if err := wrapper.CallGenerateSampleFile(exampleSize, exampleFile); err != nil {
		panic(err)
	}

	nodes := wrapper.CallCountNodeNum(exampleFile)
	fmt.Printf("%s is of %d nodes size ...\n", exampleFile, nodes)
}
