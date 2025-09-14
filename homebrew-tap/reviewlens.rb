class ReviewLens < Formula
  desc "CLI for the Intelligent Code Review Agent"
  homepage "https://github.com/Review-LensAi/reviewlens"
  version "1.0.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-aarch64-apple-darwin.tar.gz"
      sha256 "984c1d422681008b4a7d7ba7a05519cf6542b29d77b5c009836c186838d710db"
    else
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-x86_64-apple-darwin.tar.gz"
      sha256 "4272c939af27ec09cd87cbe78cedded84fc133f84bd656e3fe2827ce99933bfc"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "c845d5bcfeb8e45afeb3ffeb061dcdebff296ba9f73b3d0661726afc49afc41f"
    else
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "ae17f91b06e660145133de6ba806e2c8194835bc8bfb9e0030adcae57870d115"
    end
  end

  def install
    bin.install "reviewlens"
  end

  test do
    assert_match version.to_s, shell_output("\#{bin}/reviewlens --version")
  end
end
