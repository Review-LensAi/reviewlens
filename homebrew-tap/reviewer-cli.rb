class ReviewLens < Formula
  desc "CLI for the Intelligent Code Review Agent"
  homepage "https://github.com/Review-LensAi/reviewlens"
  version "1.0.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-aarch64-apple-darwin.tar.gz"
      sha256 "<ARM64_MAC_SHA256>"
    else
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-x86_64-apple-darwin.tar.gz"
      sha256 "<X86_64_MAC_SHA256>"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "<ARM64_LINUX_SHA256>"
    else
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "<X86_64_LINUX_SHA256>"
    end
  end

  def install
    bin.install "reviewlens"
  end

  test do
    system "\#{bin}/reviewlens", "--help"
  end
end
