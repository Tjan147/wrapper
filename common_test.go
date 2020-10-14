package wrapper

import (
	"fmt"
	"os"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestUnpaddedSpace(t *testing.T) {
	size2K := uint64(2 * 1024)
	size512M := uint64(512 * 1024 * 1024)
	size32G := uint64(32 * 1024 * 1024 * 1024)

	fmt.Printf("UnpaddedSpace(%d(2K)) = %dB\n", size2K, UnpaddedSpace(size2K))
	fmt.Printf("UnpaddedSpace(%d(512M)) = %dB\n", size512M, UnpaddedSpace(size512M))
	fmt.Printf("UnpaddedSpace(%d(32G)) = %dB\n", size32G, UnpaddedSpace(size32G))
}

func TestCreateFakeFile(t *testing.T) {
	samplePath := "./TestCreateFakeFile.dat"

	sampleSize1 := uint64(254)
	CreateFakeDataFile(samplePath, sampleSize1)
	info, err := os.Stat(samplePath)
	require.NoError(t, err)
	require.Equal(t, sampleSize1, uint64(info.Size()))

	sampleSize2 := uint64(1024)
	CreateFakeDataFile(samplePath, sampleSize2)
	info, err = os.Stat(samplePath)
	require.NoError(t, err)
	require.Equal(t, sampleSize2, uint64(info.Size()))

	sampleSize3 := uint64(2559)
	CreateFakeDataFile(samplePath, sampleSize3)
	info, err = os.Stat(samplePath)
	require.NoError(t, err)
	require.Equal(t, sampleSize3, uint64(info.Size()))

	require.NoError(t, os.Remove(samplePath))
}
