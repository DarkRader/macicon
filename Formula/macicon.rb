class Macicon < Formula
  desc "Generate, customize, and apply Apple continuous-curvature squircle app icons on macOS"
  homepage "https://github.com/DarkRader/macicon"
  version "0.1.1"
  license "MIT"
  depends_on :macos

  url "https://github.com/DarkRader/macicon/releases/download/v#{version}/macicon-v#{version}-macos-universal.tar.gz"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"

  def install
    bin.install "macicon"
  end

  test do
    assert_match "macicon #{version}", shell_output("#{bin}/macicon --version")
  end
end
