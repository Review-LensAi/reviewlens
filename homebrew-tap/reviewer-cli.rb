class ReviewerCli < Formula
  desc "CLI for the Intelligent Code Review Agent"
  homepage "https://github.com/some-org/intelligent-code-reviewer"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/some-org/intelligent-code-reviewer/releases/download/v\#{version}/reviewer-cli-aarch64-apple-darwin.tar.gz"
      sha256 "<ARM64_MAC_SHA256>"
    else
      url "https://github.com/some-org/intelligent-code-reviewer/releases/download/v\#{version}/reviewer-cli-x86_64-apple-darwin.tar.gz"
      sha256 "<X86_64_MAC_SHA256>"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/some-org/intelligent-code-reviewer/releases/download/v\#{version}/reviewer-cli-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "<ARM64_LINUX_SHA256>"
    else
      url "https://github.com/some-org/intelligent-code-reviewer/releases/download/v\#{version}/reviewer-cli-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "<X86_64_LINUX_SHA256>"
    end
  end

  def install
    bin.install "reviewer-cli"
  end

  test do
    system "\#{bin}/reviewer-cli", "--help"
  end
end
