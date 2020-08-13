package wrapper

import (
	"os"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestCallRustSetup(t *testing.T) {
	// sentinel call
	CallSentinel("tjan@golang.TestCallRustSample()")

	err := os.Mkdir("sample", 0755)
	require.NoError(t, err)

	ret := CallSetup("rust/sample/sample.txt", "sample")
	require.Equal(t, uint32(0), ret)
}
