package wrapper

import (
	"os"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestCallRustSetup(t *testing.T) {
	// sentinel call
	callRustSample("tjan@golang.TestCallRustSample()")

	err := os.Mkdir("sample", 0644)
	require.NoError(t, err)

	ret := callSetup("rust/sample/sample.txt", "sample")
	require.Equal(t, 0, ret)
}
