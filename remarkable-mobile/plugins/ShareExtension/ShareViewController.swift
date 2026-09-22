// ShareExtension/ShareViewController.swift
// iOS Share Extension for sending PDFs/EPUBs to reMarkable

import UIKit
import Social
import MobileCoreServices
import UniformTypeIdentifiers

class ShareViewController: SLComposeServiceViewController {
    
    private var fileURL: URL?
    private var fileName: String?
    private var fileType: String?
    
    override func viewDidLoad() {
        super.viewDidLoad()
        
        // Extract shared content
        extractSharedContent()
        
        // Set placeholder text
        placeholder = "Send to reMarkable"
    }
    
    override func isContentValid() -> Bool {
        // Validate that we have a valid file
        return fileURL != nil && fileName != nil
    }
    
    override func didSelectPost() {
        // User tapped Post
        guard let url = fileURL, let name = fileName else {
            extensionContext?.completeRequest(returningItems: [], completionHandler: nil)
            return
        }
        
        // Read file data
        guard let fileData = try? Data(contentsOf: url) else {
            showError("Could not read file")
            return
        }
        
        // Get server URL from app group
        let defaults = UserDefaults(suiteName: "group.com.remarkable.sync")
        guard let serverUrl = defaults?.string(forKey: "serverUrl") else {
            showError("Not connected to server. Open the app to connect.")
            return
        }
        
        // Upload file
        uploadFile(
            data: fileData,
            name: name,
            type: fileType ?? "pdf",
            serverUrl: serverUrl
        )
    }
    
    override func configurationItems() -> [Any]! {
        // Return configuration items (folder selection, etc.)
        let folderItem = SLComposeSheetConfigurationItem()!
        folderItem.title = "Folder"
        folderItem.value = "My Files"
        folderItem.tapHandler = { [weak self] in
            self?.selectFolder()
        }
        
        return [folderItem]
    }
    
    // MARK: - Private Methods
    
    private func extractSharedContent() {
        guard let extensionItems = extensionContext?.inputItems as? [NSExtensionItem] else {
            return
        }
        
        for item in extensionItems {
            guard let attachments = item.attachments else { continue }
            
            for provider in attachments {
                // Check for PDF
                if provider.hasItemConformingToTypeIdentifier(UTType.pdf.identifier) {
                    loadItem(provider: provider, typeIdentifier: UTType.pdf.identifier, fileType: "pdf")
                    return
                }
                
                // Check for EPUB
                if provider.hasItemConformingToTypeIdentifier("org.idpf.epub-container") {
                    loadItem(provider: provider, typeIdentifier: "org.idpf.epub-container", fileType: "epub")
                    return
                }
            }
        }
    }
    
    private func loadItem(provider: NSItemProvider, typeIdentifier: String, fileType: String) {
        provider.loadItem(forTypeIdentifier: typeIdentifier, options: nil) { [weak self] (item, error) in
            guard let self = self else { return }
            
            if let error = error {
                print("Error loading item: \(error)")
                return
            }
            
            var url: URL?
            
            if let itemURL = item as? URL {
                url = itemURL
            } else if let data = item as? Data {
                // Write to temp file
                let tempURL = FileManager.default.temporaryDirectory
                    .appendingPathComponent(UUID().uuidString)
                    .appendingPathExtension(fileType)
                try? data.write(to: tempURL)
                url = tempURL
            }
            
            guard let fileURL = url else { return }
            
            DispatchQueue.main.async {
                self.fileURL = fileURL
                self.fileName = fileURL.deletingPathExtension().lastPathComponent
                self.fileType = fileType
                self.validateContent()
            }
        }
    }
    
    private func uploadFile(data: Data, name: String, type: String, serverUrl: String) {
        let url = URL(string: "\(serverUrl)/sync/v3/files")!
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        
        let body: [String: Any] = [
            "name": name,
            "type": type,
            "parent": "",
            "content": data.base64EncodedString()
        ]
        
        request.httpBody = try? JSONSerialization.data(withJSONObject: body)
        
        let task = URLSession.shared.dataTask(with: request) { [weak self] _, response, error in
            DispatchQueue.main.async {
                if let error = error {
                    self?.showError("Upload failed: \(error.localizedDescription)")
                    return
                }
                
                if let httpResponse = response as? HTTPURLResponse,
                   httpResponse.statusCode >= 200 && httpResponse.statusCode < 300 {
                    self?.extensionContext?.completeRequest(returningItems: [], completionHandler: nil)
                } else {
                    self?.showError("Upload failed")
                }
            }
        }
        
        task.resume()
    }
    
    private func selectFolder() {
        // TODO: Present folder selection UI
        let alert = UIAlertController(
            title: "Select Folder",
            message: "Folder selection coming soon",
            preferredStyle: .alert
        )
        alert.addAction(UIAlertAction(title: "OK", style: .default))
        present(alert, animated: true)
    }
    
    private func showError(_ message: String) {
        let alert = UIAlertController(
            title: "Error",
            message: message,
            preferredStyle: .alert
        )
        alert.addAction(UIAlertAction(title: "OK", style: .default) { [weak self] _ in
            self?.extensionContext?.cancelRequest(withError: NSError(
                domain: "com.remarkable.sync",
                code: -1,
                userInfo: [NSLocalizedDescriptionKey: message]
            ))
        })
        present(alert, animated: true)
    }
}
