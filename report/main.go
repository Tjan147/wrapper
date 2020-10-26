package main

import (
	"bufio"
	"fmt"
	"io"
	"os"
	"path"
	"strconv"
	"strings"
	"time"

	"github.com/go-echarts/go-echarts/v2/charts"
	"github.com/go-echarts/go-echarts/v2/components"
	"github.com/go-echarts/go-echarts/v2/opts"
)

const (
	// TARGETNAME is the hard coded target file name
	TARGETNAME = "profile.txt"

	// OUTFILE is the hard coded output file name
	OUTFILE = "profile.html"
)

// TODO: fix the magic numbers here
func parseProfileLine(line string) (cpu, mem *opts.LineData, err error) {
	items := strings.Split(strings.TrimSpace(line), " ")
	if len(items) != 2 {
		return nil, nil, fmt.Errorf("malformat line: %s", line)
	}

	cpuData, err := strconv.ParseFloat(items[0], 64)
	if err != nil {
		return nil, nil, err
	}

	memData, err := strconv.ParseInt(items[1], 10, 64)
	if err != nil {
		return nil, nil, err
	}

	cpu = &opts.LineData{Value: cpuData}
	mem = &opts.LineData{Value: memData / 1024} // in MB
	return
}

func genXAxis(interval time.Duration, count int) []string {
	ret := make([]string, 0)
	// start from 1 rather than 0, see run_bench.sh script
	for i := 1; i <= count; i++ {
		dur := time.Duration(i) * interval
		ret = append(ret, fmt.Sprintf(".0%fs", dur.Seconds()))
	}

	return ret
}

func main() {
	if len(os.Args) != 3 {
		fmt.Println("Require 2 input arguments.")
		fmt.Println("Example:")
		fmt.Printf("\t%s sample 5s\n", os.Args[0])
		os.Exit(0)
	}

	dir := strings.TrimSpace(os.Args[1])

	interval, err := time.ParseDuration(strings.TrimSpace(os.Args[2]))
	if err != nil {
		panic(err)
	}

	profile, err := os.Open(path.Join(dir, TARGETNAME))
	if err != nil {
		panic(err)
	}
	defer profile.Close()

	cpuItems := make([]opts.LineData, 0)
	memItems := make([]opts.LineData, 0)
	scanner := bufio.NewScanner(profile)
	for scanner.Scan() {
		cpu, mem, err := parseProfileLine(scanner.Text())
		if err != nil {
			panic(err)
		}

		cpuItems = append(cpuItems, *cpu)
		memItems = append(memItems, *mem)
	}

	if err := scanner.Err(); err != nil {
		panic(err)
	}

	lineChart := charts.NewLine()
	lineChart.SetGlobalOptions(
		charts.WithTitleOpts(opts.Title{Title: "CPU/Memory Cost"}),
		charts.WithInitializationOpts(opts.Initialization{Theme: "shine"}),
	)

	lineChart.SetXAxis(genXAxis(interval, len(cpuItems))).
		AddSeries("CPU", cpuItems).
		AddSeries("Memory", memItems)

	page := components.NewPage()
	page.AddCharts(lineChart)

	out, err := os.Create(path.Join(dir, OUTFILE))
	if err != nil {
		panic(err)
	}
	defer out.Close()

	if err := page.Render(io.MultiWriter(out)); err != nil {
		panic(err)
	}
}
