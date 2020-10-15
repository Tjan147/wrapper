package wrapper

import (
	"crypto/rand"
	"os"

	"github.com/filecoin-project/go-state-types/abi"
)

// Statement contains necessary info for PoRep verify
type Statement struct{}

// UnpaddedSpace returns the actual effective space in Byte
func UnpaddedSpace(sectorSize uint64) abi.UnpaddedPieceSize {
	return abi.PaddedPieceSize(sectorSize).Unpadded()
}

// fakeDataFileMode for test purpose only
const (
	fakeDataFileMode = 0644
	fakeDataBuffSize = 1024
)

// CreateFakeDataFile used for test purpose only
func CreateFakeDataFile(path string, size uint64) error {
	left := size

	file, err := os.OpenFile(path, os.O_CREATE|os.O_WRONLY, fakeDataFileMode)
	if err != nil {
		return err
	}
	defer file.Close()

	buf := make([]byte, fakeDataBuffSize)
	for left >= fakeDataBuffSize {
		if _, err := rand.Read(buf); err != nil {
			return err
		}

		if _, err := file.Write(buf); err != nil {
			return err
		}

		left -= fakeDataBuffSize
	}
	if left > 1 {
		buf = make([]byte, left)

		if _, err := rand.Read(buf); err != nil {
			return err
		}

		if _, err := file.Write(buf); err != nil {
			return err
		}
	}

	return nil
}
