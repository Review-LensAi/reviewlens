class ReviewLens < Formula
  desc "CLI for the Intelligent Code Review Agent"
  homepage "https://github.com/Review-LensAi/reviewlens"
  version "1.0.1"

  # Version and sha256 values are updated automatically by
  # scripts/update-homebrew-formula.py during release preparation.
  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-aarch64-apple-darwin.tar.gz"
      sha256 "d359db211ebbd7bc4f236d09dd8e8e784f4715fb1e5d3d734f75eaf5a82900dc"
    else
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-x86_64-apple-darwin.tar.gz"
      sha256 "329a861f79766574c8dc1f04906d69d9a2e924a0ac541ef8c179b066570771d6"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "137f9776a367c48c13cf33995700183a22ff2fe2ecd565bd5b29c55d56928522"
    else
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "cb1d30a59b5ca47f70cd506fdff36c2e43598ac4f8c11e79c1ba7b18deb38cde"
    end
  end

  def install
    bin.install "reviewlens"
  end

  test do
    assert_match version.to_s, shell_output("\#{bin}/reviewlens --version")
  end
end
