package wrapper

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"time"

	"github.com/filecoin-project/go-state-types/abi"
)

// StepMeasure holds the time cost info of a single PoRep phase
type StepMeasure struct {
	Name  string        `json:"name"`
	Cost  time.Duration `json:"cost"`
	start time.Time
}

// NewStepMeasure as the factory
func NewStepMeasure(n string) *StepMeasure {
	return &StepMeasure{
		Name:  n,
		start: time.Now(),
	}
}

// Done end the step's
func (s *StepMeasure) Done() *StepMeasure {
	s.Cost = time.Now().Sub(s.start)
	return s
}

// Report holds the step measure information for a whole PoRep procedure
type Report struct {
	Detail     string         `json:"detail"`
	SectorSize string         `json:"sector_size"`
	Steps      []*StepMeasure `json:"steps"`
}

// NewReport as the factory
func NewReport(detail string, typ abi.RegisteredSealProof) (*Report, error) {
	sectorSize, err := typ.SectorSize()
	if err != nil {
		return nil, err
	}

	return &Report{
		Detail:     detail,
		SectorSize: sectorSize.ShortString(),
	}, nil
}

// AddStep add a *DONE* step measure instance to the steps list
func (r *Report) AddStep(s *StepMeasure) {
	r.Steps = append(r.Steps, s)
}

// Dump push the json formated report to the disk
func (r *Report) Dump() error {
	content, err := json.Marshal(r)
	if err != nil {
		return err
	}

	return ioutil.WriteFile(
		fmt.Sprintf("%s-%s.json", r.Detail, r.SectorSize),
		content,
		0644,
	)
}
