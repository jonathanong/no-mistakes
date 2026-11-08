#include <algorithm>
#include <charconv>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <unordered_map>
#include <unordered_set>

struct History { uint64_t live = 0, maximum = 0, bytes = 0; };
uint64_t number(std::string_view &s) {
    auto start = s.find_first_not_of(" \t\r");
    if (start == s.npos) throw std::runtime_error("missing field");
    s.remove_prefix(start);
    auto end = s.find_first_of(" \t\r");
    auto token = s.substr(0, end);
    uint64_t value;
    auto result = std::from_chars(token.data(), token.data() + token.size(), value, 16);
    if (result.ec != std::errc() || result.ptr != token.data() + token.size())
        throw std::runtime_error("invalid hex field");
    s.remove_prefix(end == s.npos ? s.size() : end);
    return value;
}
struct Event { char kind; uint64_t bytes, pointer; };
Event event(const std::string &line, uint64_t frames) {
    std::string_view s(line); char kind = s[0]; s.remove_prefix(1);
    uint64_t bytes = 0;
    if (kind == '+') {
        bytes = number(s);
        auto trace = number(s);
        if (!trace || trace > frames) throw std::runtime_error("unknown trace");
    }
    auto pointer = number(s);
    if (s.find_first_not_of(" \t\r") != s.npos) throw std::runtime_error("excess field");
    return {kind, bytes, pointer};
}
int main(int argc, char **argv) {
    try {
        if (argc != 4) throw std::runtime_error("expected input output audit");
        std::ifstream input(argv[1]);
        if (!input) throw std::runtime_error("input open");
        std::unordered_map<uint64_t, History> history;
        std::unordered_set<uint64_t> excluded;
        uint64_t duplicates = 0, unknown = 0, frames = 0, allocations = 0, frees = 0;
        std::string line;
        while (std::getline(input, line)) {
            if (line.empty()) continue;
            if (line[0] == 't') { ++frames; continue; }
            if (line[0] != '+' && line[0] != '-') continue;
            auto e = event(line, frames);
            auto &h = history[e.pointer];
            if (e.kind == '+') {
                ++allocations;
                if (h.live) { ++duplicates; excluded.insert(e.pointer); }
                ++h.live;
                h.maximum = std::max(h.maximum, h.live);
                h.bytes = std::max(h.bytes, e.bytes);
            } else {
                ++frees;
                if (h.live) --h.live;
                else { ++unknown; excluded.insert(e.pointer); }
            }
        }
        if (!input.eof()) throw std::runtime_error("input read");
        uint64_t bound = 0;
        for (auto pointer : excluded) {
            const auto &h = history.at(pointer);
            if (h.maximum && h.bytes > UINT64_MAX / h.maximum) throw std::runtime_error("bound overflow");
            auto contribution = h.maximum * h.bytes;
            if (contribution > UINT64_MAX - bound) throw std::runtime_error("sum overflow");
            bound += contribution;
        }
        input.clear(); input.seekg(0);
        std::ofstream output(argv[2]);
        if (!output) throw std::runtime_error("output open");
        uint64_t omitted_allocations = 0, omitted_frees = 0;
        frames = 0;
        while (std::getline(input, line)) {
            if (!line.empty() && line[0] == 't') ++frames;
            if (!line.empty() && (line[0] == '+' || line[0] == '-')) {
                auto e = event(line, frames);
                if (excluded.contains(e.pointer)) {
                    if (e.kind == '+') ++omitted_allocations; else ++omitted_frees;
                    continue;
                }
            }
            output << line << '\n';
        }
        output.close();
        if (!input.eof() || !output) throw std::runtime_error("filtered I/O");
        std::ofstream audit(argv[3]);
        audit << "{\"duplicateEvents\":" << duplicates << ",\"unknownFrees\":" << unknown
              << ",\"quarantinedAddresses\":" << excluded.size() << ",\"payloadUpperBound\":" << bound
              << ",\"totalAllocations\":" << allocations << ",\"totalFrees\":" << frees
              << ",\"omittedAllocations\":" << omitted_allocations << ",\"omittedFrees\":" << omitted_frees << "}\n";
        if (!audit) throw std::runtime_error("audit write");
    } catch (const std::exception &error) { std::cerr << error.what() << '\n'; return 2; }
}
