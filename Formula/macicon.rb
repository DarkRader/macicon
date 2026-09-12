class Macicon < Formula
  desc "Generate, customize, and apply Apple continuous-curvature squircle app icons on macOS"
  homepage "https://github.com/DarkRader/macicon"
  version "0.2.2"
  license "MIT"
  depends_on :macos

  url "https://github.com/DarkRader/macicon/releases/download/v#{version}/macicon-v#{version}-macos-universal.tar.gz"
  sha256 "61c50f1a56cb5a068d83cecfb7401dd74a2bbd974ec2763f76af2258382ac559"

  def install
    bin.install "macicon"
  end

  test do
    assert_match "macicon #{version}", shell_output("#{bin}/macicon --version")
  end
end
