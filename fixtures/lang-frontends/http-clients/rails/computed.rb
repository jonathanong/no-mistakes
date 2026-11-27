Faraday.get(path)
HTTParty.get "/wrong" + path
HTTParty.get "/wrong/#{path}"
Net::HTTP.get(URI("http://localhost/wrong") + path)
