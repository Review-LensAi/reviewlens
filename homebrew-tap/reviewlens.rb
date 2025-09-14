class ReviewLens < Formula
  desc "CLI for the Intelligent Code Review Agent"
  homepage "https://github.com/Review-LensAi/reviewlens"
  version "1.0.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-aarch64-apple-darwin.tar.gz"
      sha256 "f45f78b42491fa28a6d41a04ad488d75fc616a958182cb12794977152b035ac3"
    else
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-x86_64-apple-darwin.tar.gz"
      sha256 "f45f78b42491fa28a6d41a04ad488d75fc616a958182cb12794977152b035ac3"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "f45f78b42491fa28a6d41a04ad488d75fc616a958182cb12794977152b035ac3"
    else
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "f45f78b42491fa28a6d41a04ad488d75fc616a958182cb12794977152b035ac3"
    end
  end

  def install
    bin.install "reviewlens"
  end

  test do
    assert_match version.to_s, shell_output("\#{bin}/reviewlens --version")
  end
end
