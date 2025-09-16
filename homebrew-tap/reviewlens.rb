class ReviewLens < Formula
  desc "CLI for the Intelligent Code Review Agent"
  homepage "https://github.com/Review-LensAi/reviewlens"
  version "1.0.1"

  # The sha256 placeholders below are updated by
  # scripts/update-homebrew-formula.py during release.
  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-aarch64-apple-darwin.tar.gz"
      sha256 "b5233e197933888d246ebaa700d0ff8142f83087c511b96dec2dc5c37dd4f281"
    else
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-x86_64-apple-darwin.tar.gz"
      sha256 "dc8681b0e35a0fb8f6eca24faae7d5e209945afd8f79adc4e9a2b6f565a33804"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "cd0659f509d049a4d7195967b22fa6ea1b8b403e26e1a62d1d55fba2086907dd"
    else
      url "https://github.com/Review-LensAi/reviewlens/releases/download/v\#{version}/reviewlens-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "7ecc0d49d4112fe9afd6f6bf9b3095aab1925d502168ee415b80c35fd13a4d73"
    end
  end

  def install
    bin.install "reviewlens"
  end

  test do
    assert_match version.to_s, shell_output("\#{bin}/reviewlens --version")
  end
end
