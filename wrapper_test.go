package wrapper

import (
	"testing"
)

func TestCallRustSentinel(t *testing.T) {
	// sentinel call
	CallSentinel("tjan@golang.TestCallRustSample()")
}
