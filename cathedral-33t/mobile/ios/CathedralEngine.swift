import Foundation

public class CathedralEngine {
    private let handle: UnsafeMutableRawPointer

    public init() {
        // handle = cathedral_engine_new()
        handle = UnsafeMutableRawPointer(bitPattern: 0)!
    }

    deinit {
        // cathedral_engine_free(handle)
    }

    public func loadModel(path: String) -> Bool {
        // return cathedral_engine_load_model(handle, path)
        return true
    }

    public func infer(input: String) -> String? {
        return nil
    }

    public func inferFloat(input: [Float]) -> [Float]? {
        var result = [Float]()
        return result
    }

    public func reset() {
        // cathedral_engine_reset(handle)
    }
}
