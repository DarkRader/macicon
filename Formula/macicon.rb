class Macicon < Formula
  desc "Generate, customize, and apply Apple continuous-curvature squircle app icons on macOS"
  homepage "https://github.com/DarkRader/macicon"
  version "0.2.3"
  license "MIT"
  depends_on :macos

  url "https://github.com/DarkRader/macicon/releases/download/v#{version}/macicon-v#{version}-macos-universal.tar.gz"
  sha256 "af9090944c52dffe9bc533be9f035df95ad293d8cef56f2760472800b8069374"

  def install
    bin.install "macicon"
  end

  test do
    assert_match "macicon #{version}", shell_output("#{bin}/macicon --version")
  end
end
